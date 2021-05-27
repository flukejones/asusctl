use notify_rust::{Hint, Notification, NotificationHandle};
use rog_aura::AuraEffect;
use rog_dbus::{DbusProxies, Signals};
use rog_profiles::profiles::{FanLevel, Profile};
use rog_types::gfx_vendors::GfxVendors;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

const NOTIF_HEADER: &str = "ROG Control";

macro_rules! notify {
    ($notifier:ident, $last_notif:ident, $data:expr) => {
        if let Some(notif) = $last_notif.take() {
            notif.close();
        }
        if let Ok(x) = $notifier($data) {
            $last_notif = Some(x);
        }
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("asus-notify version {}", env!("CARGO_PKG_VERSION"));
    println!("   rog-dbus version {}", rog_dbus::VERSION);

    let (proxies, conn) = DbusProxies::new()?;
    let signals = Signals::new(&proxies)?;

    let mut last_notification: Option<NotificationHandle> = None;

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

        if let Ok(data) = signals.led_mode.try_recv() {
            notify!(do_led_notif, last_notification, &data);
        }
        if let Ok(data) = signals.profile.try_recv() {
            notify!(do_thermal_notif, last_notification, &data);
        }
        if let Ok(data) = signals.charge.try_recv() {
            notify!(do_charge_notif, last_notification, &data);
        }
        if let Ok(data) = signals.gfx_vendor.try_recv() {
            notify!(do_gfx_notif, last_notification, &data);
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

macro_rules! base_notification {
    ($body:expr) => {
        Notification::new()
            .summary(NOTIF_HEADER)
            .body($body)
            .timeout(2000)
            .show()
    };
}

fn do_led_notif(ledmode: &AuraEffect) -> Result<NotificationHandle, notify_rust::error::Error> {
    base_notification!(&format!(
        "Keyboard LED mode changed to {}",
        ledmode.mode_name()
    ))
}

fn do_charge_notif(limit: &u8) -> Result<NotificationHandle, notify_rust::error::Error> {
    base_notification!(&format!("Battery charge limit changed to {}", limit))
}

fn do_gfx_notif(vendor: &GfxVendors) -> Result<NotificationHandle, notify_rust::error::Error> {
    base_notification!(&format!(
        "Graphics mode changed to {}",
        <&str>::from(vendor)
    ))
}
