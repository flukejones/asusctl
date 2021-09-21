use notify_rust::{Hint, Notification, NotificationHandle};
use rog_aura::AuraEffect;
use rog_dbus::{DbusProxies, Signals};
use rog_profiles::Profile;
use std::error::Error;
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;
use std::{process, thread};
use supergfxctl::gfx_vendors::{GfxRequiredUserAction, GfxVendors};
use supergfxctl::zbus_proxy::GfxProxy;
use zbus::Connection;

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

macro_rules! base_notification {
    ($body:expr) => {
        Notification::new()
            .summary(NOTIF_HEADER)
            .body($body)
            .timeout(2000)
            .show()
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("asus-notify version {}", env!("CARGO_PKG_VERSION"));
    println!("   rog-dbus version {}", rog_dbus::VERSION);
    println!("supergfxctl version {}", supergfxctl::VERSION);

    let (proxies, conn) = DbusProxies::new()?;
    let signals = Signals::new(&proxies)?;

    let mut last_notification: Option<NotificationHandle> = None;

    let recv = proxies.setup_recv(conn);
    let mut err_count = 0;

    gfx_thread()?;

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
    }
}

fn gfx_thread() -> Result<(), Box<dyn std::error::Error>> {
    let mut last_notification: Option<NotificationHandle> = None;

    let conn = Connection::new_system()?;
    let proxy = GfxProxy::new(&conn)?;

    let (tx1, rx1) = channel();
    proxy.connect_notify_gfx(tx1)?;

    let (tx2, rx2) = channel();
    proxy.connect_notify_action(tx2)?;

    thread::spawn(move || loop {
        if proxy
            .next_signal()
            .map_err(|e| println!("Error: {}", e))
            .is_err()
        {
            break;
        }

        if let Ok(data) = rx1.try_recv() {
            notify!(do_gfx_notif, last_notification, &data);
        }

        if let Ok(data) = rx2.try_recv() {
            match data {
                GfxRequiredUserAction::Logout | GfxRequiredUserAction::Reboot => {
                    do_gfx_action_notif(&data)
                        .map_err(|e| {
                            println!("Error: {}", e);
                        })
                        .ok();
                }
                GfxRequiredUserAction::Integrated => {
                    base_notification!(
                        "You must be in integrated mode first to switch to the requested mode"
                    )
                    .map_err(|e| {
                        println!("Error: {}", e);
                    })
                    .ok();
                }
                GfxRequiredUserAction::None => {}
            }
        }
    });
    Ok(())
}

fn do_thermal_notif(profile: &Profile) -> Result<NotificationHandle, Box<dyn Error>> {
    let icon = match profile {
        Profile::Balanced => "asus_notif_yellow",
        Profile::Performance => "asus_notif_red",
        Profile::Quiet => "asus_notif_green",
    };
    let profile: &str = (*profile).into();
    let x = Notification::new()
        .summary("ASUS ROG")
        .body(&format!(
            "Thermal profile changed to {}",
            profile.to_uppercase(),
        ))
        .hint(Hint::Resident(true))
        .timeout(2000)
        .hint(Hint::Category("device".into()))
        //.hint(Hint::Transient(true))
        .icon(icon)
        .show()?;
    Ok(x)
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
    let icon = match vendor {
        GfxVendors::Nvidia => "/usr/share/icons/hicolor/scalable/status/gpu-nvidia.svg",
        GfxVendors::Integrated => "/usr/share/icons/hicolor/scalable/status/gpu-integrated.svg",
        GfxVendors::Compute => "/usr/share/icons/hicolor/scalable/status/gpu-compute.svg",
        GfxVendors::Vfio => "/usr/share/icons/hicolor/scalable/status/gpu-vfio.svg",
        GfxVendors::Hybrid => "/usr/share/icons/hicolor/scalable/status/gpu-hybrid.svg",
    };
    Notification::new()
        .summary(NOTIF_HEADER)
        .body(&format!(
            "Graphics mode changed to {}",
            <&str>::from(vendor)
        ))
        .timeout(2000)
        .icon(icon)
        .show()
}

fn do_gfx_action_notif(vendor: &GfxRequiredUserAction) -> Result<(), notify_rust::error::Error> {
    let mut notif = Notification::new()
        .summary(NOTIF_HEADER)
        .timeout(2000)
        .urgency(notify_rust::Urgency::Critical)
        .icon("/usr/share/icons/hicolor/scalable/status/notification-reboot.svg")
        .finalize();

    if matches!(vendor, GfxRequiredUserAction::Logout) {
        notif.action("logout", "Logout now?");
    } else if matches!(vendor, GfxRequiredUserAction::Reboot) {
        notif.action("reboot", "Reboot now?");
    }

    notif.body("Graphics mode changed");
    notif.show()?.wait_for_action(|action| match action {
        "logout" => {
            let r = process::Command::new("gnome-session-quit").spawn();
            if r.is_err() {
                // Try to prompt plasma logout screen via dbus
                let conn = Connection::new_session().unwrap();
                conn.call_method(
                    Some(&"org.kde.LogoutPrompt"),
                    &"/LogoutPrompt",
                    Some(&"org.kde.LogoutPrompt"),
                    &"promptLogout",
                    &(),
                ).ok();
            }
        }
        "reboot" => {
            process::Command::new("systemctl")
                .arg("reboot")
                .spawn()
                .ok();
        }
        _ => (),
    });
    Ok(())
}
