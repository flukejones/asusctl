use notify_rust::{Hint, Notification, NotificationHandle};
use rog_aura::AuraEffect;
use rog_dbus::{
    zbus_led::LedProxy, zbus_platform::RogBiosProxy, zbus_power::PowerProxy,
    zbus_profile::ProfileProxy,
};
use rog_profiles::Profile;
use smol::{future, Executor};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use zbus::export::futures_util::StreamExt;

const NOTIF_HEADER: &str = "ROG Control";

macro_rules! notify {
    ($notifier:ident, $last_notif:ident, $data:expr) => {
        if let Some(notif) = $last_notif.take() {
            notif.close();
        }
        if let Ok(x) = $notifier($data) {
            $last_notif.replace(x);
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

type SharedHandle = Arc<Mutex<Option<NotificationHandle>>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("asus-notify version {}", env!("CARGO_PKG_VERSION"));
    println!("   rog-dbus version {}", rog_dbus::VERSION);

    let last_notification: SharedHandle = Arc::new(Mutex::new(None));

    let executor = Executor::new();
    // BIOS notif
    let x = last_notification.clone();
    executor
        .spawn(async move {
            let conn = zbus::Connection::system().await.unwrap();
            let proxy = RogBiosProxy::new(&conn).await.unwrap();
            if let Ok(p) = proxy.receive_notify_post_boot_sound().await {
                p.for_each(|e| {
                    if let Ok(out) = e.args() {
                        if let Ok(ref mut lock) = x.try_lock() {
                            notify!(do_post_sound_notif, lock, &out.on());
                        }
                    }
                    future::ready(())
                })
                .await;
            };
        })
        .detach();

    // Charge notif
    let x = last_notification.clone();
    executor
        .spawn(async move {
            let conn = zbus::Connection::system().await.unwrap();
            let proxy = PowerProxy::new(&conn).await.unwrap();
            if let Ok(p) = proxy.receive_notify_charge_control_end_threshold().await {
                p.for_each(|e| {
                    if let Ok(out) = e.args() {
                        if let Ok(ref mut lock) = x.try_lock() {
                            notify!(do_charge_notif, lock, &out.limit);
                        }
                    }
                    future::ready(())
                })
                .await;
            };
        })
        .detach();

    // Profile notif
    let x = last_notification.clone();
    executor
        .spawn(async move {
            let conn = zbus::Connection::system().await.unwrap();
            let proxy = ProfileProxy::new(&conn).await.unwrap();
            if let Ok(p) = proxy.receive_notify_profile().await {
                p.for_each(|e| {
                    if let Ok(out) = e.args() {
                        if let Ok(ref mut lock) = x.try_lock() {
                            notify!(do_thermal_notif, lock, &out.profile);
                        }
                    }
                    future::ready(())
                })
                .await;
            };
        })
        .detach();

    // LED notif
    executor
        .spawn(async move {
            let conn = zbus::Connection::system().await.unwrap();
            let proxy = LedProxy::new(&conn).await.unwrap();
            if let Ok(p) = proxy.receive_notify_led().await {
                p.for_each(|e| {
                    if let Ok(out) = e.args() {
                        if let Ok(ref mut lock) = last_notification.try_lock() {
                            notify!(do_led_notif, lock, &out.data);
                        }
                    }
                    future::ready(())
                })
                .await;
            };
        })
        .detach();

    loop {
        smol::block_on(executor.tick());
    }
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

fn do_post_sound_notif(on: &bool) -> Result<NotificationHandle, notify_rust::error::Error> {
    base_notification!(&format!("BIOS Post sound {}", on))
}
