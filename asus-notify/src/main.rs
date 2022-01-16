use futures::{executor::ThreadPool, StreamExt};
use notify_rust::{Hint, Notification, NotificationHandle};
use rog_aura::AuraEffect;
use rog_dbus::{
    zbus_charge::ChargeProxy, zbus_led::LedProxy, zbus_profile::ProfileProxy,
    zbus_rogbios::RogBiosProxy,
};
use rog_profiles::Profile;
use std::{
    error::Error,
    future,
    sync::{Arc, Mutex},
};

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

    let thread_pool = ThreadPool::new()?;
    // BIOS notif
    let x = last_notification.clone();
    thread_pool.spawn_ok(async move {
        let conn = zbus::Connection::system().await.unwrap();
        let proxy = RogBiosProxy::new(&conn).await.unwrap();
        if let Ok(p) = proxy.receive_notify_post_boot_sound().await {
            p.for_each(|e| {
                if let Ok(out) = e.args() {
                    if let Ok(ref mut lock) = x.try_lock() {
                        notify!(do_post_sound_notif, lock, &out.sound());
                    }
                }
                future::ready(())
            })
            .await;
        };
    });

    // Charge notif
    let x = last_notification.clone();
    thread_pool.spawn_ok(async move {
        let conn = zbus::Connection::system().await.unwrap();
        let proxy = ChargeProxy::new(&conn).await.unwrap();
        if let Ok(p) = proxy.receive_notify_charge().await {
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
    });

    // Profile notif
    let x = last_notification.clone();
    thread_pool.spawn_ok(async move {
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
    });

    // LED notif
    thread_pool.spawn_ok(async move {
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
    });

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        //     if err_count < 3 {
        //         err_count += 1;
        //         println!("{}", err);
        //     }
        //     if err_count == 3 {
        //         err_count += 1;
        //         println!("Max error count reached. Spooling silently.");
        //     }
        //     sleep(Duration::from_millis(2000));
        //     continue;
        // }
        // err_count = 0;

        // if let Ok(data) = signals.led_mode.try_recv() {
        //     notify!(do_led_notif, last_notification, &data);
        // }
        // if let Ok(data) = signals.profile.try_recv() {
        //     notify!(do_thermal_notif, last_notification, &data);
        // }
        // if let Ok(data) = signals.charge.try_recv() {
        //     notify!(do_charge_notif, last_notification, &data);
        // }
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
