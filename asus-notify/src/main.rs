use notify_rust::{Hint, Notification, NotificationHandle};
use rog_dbus::{DbusProxies, Signals};
use rog_types::profile::Profile;
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

        if let Ok(mut lock) = signals.gfx_vendor.lock() {
            if let Some(vendor) = lock.take() {
                if let Some(notif) = last_gfx_notif.take() {
                    notif.close();
                }
                let x = do_notif(&format!(
                    "Graphics mode changed to {}",
                    <&str>::from(vendor)
                ))?;
                last_gfx_notif = Some(x);
            }
        }

        if let Ok(mut lock) = signals.charge.lock() {
            if let Some(limit) = lock.take() {
                if let Some(notif) = last_chrg_notif.take() {
                    notif.close();
                }
                let x = do_notif(&format!("Battery charge limit changed to {}", limit))?;
                last_chrg_notif = Some(x);
            }
        }

        if let Ok(mut lock) = signals.profile.lock() {
            if let Some(profile) = lock.take() {
                if let Some(notif) = last_profile_notif.take() {
                    notif.close();
                }
                if let Ok(profile) = serde_json::from_str(&profile) {
                    let profile: Profile = profile;
                    if let Ok(name) = proxies.profile().active_profile_name() {
                        let x = do_thermal_notif(&profile, &name)?;
                        last_profile_notif = Some(x);
                    }
                }
            }
        }

        if let Ok(mut lock) = signals.led_mode.lock() {
            if let Some(ledmode) = lock.take() {
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
}

fn do_thermal_notif(profile: &Profile, label: &str) -> Result<NotificationHandle, Box<dyn Error>> {
    let fan = profile.fan_preset;
    let turbo = if profile.turbo { "enabled" } else { "disabled" };
    let icon = match fan {
        0 => "asus_notif_yellow",
        1 => "asus_notif_red",
        2 => "asus_notif_green",
        _ => "asus_notif_red",
    };
    let x = Notification::new()
        .summary("ASUS ROG")
        .body(&format!(
            "Thermal profile changed to {}, turbo {}",
            label.to_uppercase(),
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
