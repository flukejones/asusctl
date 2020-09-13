use asus_nb::core_dbus::CtrlSignals;
use daemon::config::{Config, Profile};
use dbus::blocking::Connection;
use notify_rust::{Hint, Notification, NotificationHandle};
use std::error::Error;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Config::read_new()?;
    let mut last_profile = String::new();

    let connection = Connection::new_system()?;
    let signals = CtrlSignals::new(&connection)?;

    let mut last_profile_notif: Option<NotificationHandle> = None;
    let mut last_led_notif: Option<NotificationHandle> = None;
    let mut last_gfx_notif: Option<NotificationHandle> = None;
    let mut last_chrg_notif: Option<NotificationHandle> = None;

    loop {
        std::thread::sleep(Duration::from_millis(100));
        connection.process(std::time::Duration::from_millis(200))?;

        if let Ok(mut lock) = signals.gfx_signal.lock() {
            if let Some(vendor) = lock.take() {
                if let Some(notif) = last_gfx_notif.take() {
                    notif.close();
                }
                let x = do_notif(&format!("Graphics mode changed to {}", vendor))?;
                last_gfx_notif = Some(x);
            }
        }

        if let Ok(mut lock) = signals.charge_signal.lock() {
            if let Some(limit) = lock.take() {
                if let Some(notif) = last_chrg_notif.take() {
                    notif.close();
                }
                let x = do_notif(&format!("Battery charge limit changed to {}", limit))?;
                last_led_notif = Some(x);
            }
        }

        if let Ok(mut lock) = signals.ledmode_signal.lock() {
            if let Some(ledmode) = lock.take() {
                if let Some(notif) = last_led_notif.take() {
                    notif.close();
                }
                let x = do_notif(&format!(
                    "Keyboard LED mode changed to {}",
                    <&str>::from(&ledmode)
                ))?;
                last_led_notif = Some(x);
            }
        }

        // We need to do the config read because of a limitation preventing
        // easy dbus notification from the profile controller
        cfg.read();
        if last_profile != cfg.active_profile {
            if let Some(notif) = last_profile_notif.take() {
                notif.close();
            }
            if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                let x = do_thermal_notif(&profile, &cfg.active_profile)?;
                last_profile_notif = Some(x);
                last_profile = cfg.active_profile.clone();
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
        .body(&format!("Thermal profile changed to {}, turbo {}", label.to_uppercase(), turbo))
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
