use notify_rust::{Hint, Notification, NotificationHandle};
use rog_dbus::{DbusProxies, Signals};
use rog_profiles::profiles::{FanLevel, Profile};
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("asus-notify version {}", env!("CARGO_PKG_VERSION"));
    println!("     daemon version {}", daemon::VERSION);
    println!("   rog-dbus version {}", rog_dbus::VERSION);

    let (proxies, conn) = DbusProxies::new()?;
    let signals = Signals::new(&proxies)?;

    let mut last_profile_notif: Option<NotificationHandle> = None;
    let mut last_led_notif: Option<NotificationHandle> = None;
    let mut last_gfx_notif: Option<NotificationHandle> = None;
    let mut last_chrg_notif: Option<NotificationHandle> = None;

    let recv = proxies.setup_recv(conn);
    let mut err_count = 0;
    loop {
        sleep(Duration::from_millis(100));
        if let Err(err) = recv.next_signal() {
            if err_count < 3 {
                err_count += 1;
                println!("{}", err);
            }
            if err_count == 3 {
                err_count += 1;
                println!("Max error count reached. Spooling silently.");
            }
            sleep(Duration::from_millis(2000));
            continue;
        }
        err_count = 0;

        if let Ok(vendor) = signals.gfx_vendor.try_recv() {
            if let Some(notif) = last_gfx_notif.take() {
                notif.close();
            }
            let x = do_notif(&format!(
                "Graphics mode changed to {}",
                <&str>::from(vendor)
            ))?;
            last_gfx_notif = Some(x);
        }

        if let Ok(limit) = signals.charge.try_recv() {
            if let Some(notif) = last_chrg_notif.take() {
                notif.close();
            }
            let x = do_notif(&format!("Battery charge limit changed to {}", limit))?;
            last_chrg_notif = Some(x);
        }

        if let Ok(profile) = signals.profile.try_recv() {
            if let Some(notif) = last_profile_notif.take() {
                notif.close();
            }
            let x = do_thermal_notif(&profile)?;
            last_profile_notif = Some(x);
        }

        if let Ok(ledmode) = signals.led_mode.try_recv() {
            if let Some(notif) = last_led_notif.take() {
                notif.close();
            }
            let x = do_notif(&format!(
                "Keyboard LED mode changed to {}",
                ledmode.mode_name()
            ))?;
            last_led_notif = Some(x);
        }
    }
}

fn do_thermal_notif(profile: &Profile) -> Result<NotificationHandle, Box<dyn Error>> {
    let fan = profile.fan_preset;
    let turbo = if profile.turbo { "enabled" } else { "disabled" };
    let icon = match fan {
        FanLevel::Normal => "asus_notif_yellow",
        FanLevel::Boost => "asus_notif_red",
        FanLevel::Silent => "asus_notif_green",
    };
    let x = Notification::new()
        .summary("ASUS ROG")
        .body(&format!(
            "Thermal profile changed to {}, turbo {}",
            profile.name.to_uppercase(),
            turbo
        ))
        .hint(Hint::Resident(true))
        .timeout(2000)
        .hint(Hint::Category("device".into()))
        //.hint(Hint::Transient(true))
        .icon(icon)
        .show()?;
    Ok(x)
}

fn do_notif(body: &str) -> Result<NotificationHandle, Box<dyn Error>> {
    let x = Notification::new()
        .summary("ASUS ROG")
        .body(body)
        .timeout(2000)
        .show()?;
    Ok(x)
}
