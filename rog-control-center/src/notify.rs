use notify_rust::{Hint, Notification, NotificationHandle};
use rog_dbus::{
    zbus_anime::AnimeProxy, zbus_led::LedProxy, zbus_platform::RogBiosProxy,
    zbus_power::PowerProxy, zbus_profile::ProfileProxy,
};
use rog_profiles::Profile;
use smol::{future, Executor};
use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::spawn,
};
use supergfxctl::pci_device::GfxPower;
use zbus::export::futures_util::StreamExt;
use crate::error::Result;

const NOTIF_HEADER: &str = "ROG Control";

macro_rules! notify {
    ($notifier:expr, $last_notif:ident) => {
        if let Some(notif) = $last_notif.take() {
            notif.close();
        }
        if let Ok(x) = $notifier {
            $last_notif.replace(x);
        }
    };
}

macro_rules! recv_notif {
    ($executor:ident,
        $proxy:ident,
        $signal:ident,
        $was_notified:ident,
        $last_notif:ident,
        $notif_enabled:ident,
        [$($out_arg:ident)+],
        $msg:literal,
        $notifier:ident) => {
        let last_notif = $last_notif.clone();
        let notifs_enabled1 = $notif_enabled.clone();
        let notified = $was_notified.clone();
        // TODO: make a macro or generic function or something...
        $executor
            .spawn(async move {
                let conn = zbus::Connection::system().await.unwrap();
                let proxy = $proxy::new(&conn).await.unwrap();
                if let Ok(p) = proxy.$signal().await {
                    p.for_each(|e| {
                        if let Ok(out) = e.args() {
                            if notifs_enabled1.load(Ordering::SeqCst) {
                                if let Ok(ref mut lock) = last_notif.try_lock() {
                                    notify!($notifier($msg, &out$(.$out_arg)+()), lock);
                                }
                            }
                            notified.store(true, Ordering::SeqCst);
                        }
                        future::ready(())
                    })
                    .await;
                };
            })
            .detach();
    };
}

type SharedHandle = Arc<Mutex<Option<NotificationHandle>>>;

pub fn start_notifications(
    charge_notified: Arc<AtomicBool>,
    bios_notified: Arc<AtomicBool>,
    aura_notified: Arc<AtomicBool>,
    anime_notified: Arc<AtomicBool>,
    profiles_notified: Arc<AtomicBool>,
    _fans_notified: Arc<AtomicBool>,
    notifs_enabled: Arc<AtomicBool>,
) -> Result<()> {
    let last_notification: SharedHandle = Arc::new(Mutex::new(None));

    let executor = Executor::new();
    // BIOS notif
    recv_notif!(
        executor,
        RogBiosProxy,
        receive_notify_post_boot_sound,
        bios_notified,
        last_notification,
        notifs_enabled,
        [on],
        "BIOS Post sound",
        do_notification
    );

    recv_notif!(
        executor,
        RogBiosProxy,
        receive_notify_panel_od,
        bios_notified,
        last_notification,
        notifs_enabled,
        [overdrive],
        "Panel Overdrive enabled:",
        do_notification
    );

    recv_notif!(
        executor,
        RogBiosProxy,
        receive_notify_dgpu_disable,
        bios_notified,
        last_notification,
        notifs_enabled,
        [disable],
        "BIOS dGPU disabled",
        do_notification
    );

    recv_notif!(
        executor,
        RogBiosProxy,
        receive_notify_egpu_enable,
        bios_notified,
        last_notification,
        notifs_enabled,
        [enable],
        "BIOS eGPU enabled",
        do_notification
    );

    recv_notif!(
        executor,
        RogBiosProxy,
        receive_notify_gpu_mux_mode,
        bios_notified,
        last_notification,
        notifs_enabled,
        [mode],
        "BIOS GPU MUX mode (reboot required)",
        do_notification
    );

    // Charge notif
    recv_notif!(
        executor,
        PowerProxy,
        receive_notify_charge_control_end_threshold,
        charge_notified,
        last_notification,
        notifs_enabled,
        [limit],
        "Battery charge limit changed to",
        do_notification
    );

    recv_notif!(
        executor,
        PowerProxy,
        receive_notify_mains_online,
        bios_notified,
        last_notification,
        notifs_enabled,
        [on],
        "AC Power power is",
        ac_power_notification
    );

    // Profile notif
    recv_notif!(
        executor,
        ProfileProxy,
        receive_notify_profile,
        profiles_notified,
        last_notification,
        notifs_enabled,
        [profile],
        "Profile changed to",
        do_thermal_notif
    );
    // notify!(do_thermal_notif(&out.profile), lock);

    // LED notif
    recv_notif!(
        executor,
        LedProxy,
        receive_notify_led,
        aura_notified,
        last_notification,
        notifs_enabled,
        [data mode_name],
        "Keyboard LED mode changed to",
        do_notification
    );

    executor
        .spawn(async move {
            let conn = zbus::Connection::system().await.unwrap();
            let proxy = LedProxy::new(&conn).await.unwrap();
            if let Ok(p) = proxy.receive_all_signals().await {
                p.for_each(|_| {
                    aura_notified.store(true, Ordering::SeqCst);
                    future::ready(())
                })
                .await;
            };
        })
        .detach();

    executor
        .spawn(async move {
            let conn = zbus::Connection::system().await.unwrap();
            let proxy = AnimeProxy::new(&conn).await.unwrap();
            if let Ok(p) = proxy.receive_power_states().await {
                p.for_each(|_| {
                    anime_notified.store(true, Ordering::SeqCst);
                    future::ready(())
                })
                .await;
            };
        })
        .detach();

    let notifs_enabled1 = notifs_enabled.clone();
    let last_notif = last_notification.clone();
    let bios_notified1 = bios_notified.clone();
    executor
        .spawn(async move {
            let conn = zbus::Connection::system().await.unwrap();
            let proxy = supergfxctl::zbus_proxy::DaemonProxy::new(&conn)
                .await
                .unwrap();
            if let Ok(p) = proxy.receive_notify_gfx_status().await {
                p.for_each(|e| {
                    if let Ok(out) = e.args() {
                        if notifs_enabled1.load(Ordering::SeqCst) {
                            let status = out.status();
                            if *status != GfxPower::Unknown {
                                // Required check because status cycles through active/unknown/suspended
                                if let Ok(ref mut lock) = last_notif.try_lock() {
                                    notify!(
                                        do_notification(
                                            "dGPU status changed:",
                                            &format!("{status:?}",)
                                        ),
                                        lock
                                    );
                                }
                            }
                        }
                    }
                    bios_notified1.store(true, Ordering::SeqCst);
                    future::ready(())
                })
                .await;
            };
        })
        .detach();

    spawn(move || loop {
        smol::block_on(executor.tick());
    });
    Ok(())
}

fn base_notification<T>(message: &str, data: &T) -> Notification
where
    T: Display,
{
    let mut notif = Notification::new();

    notif
        .summary(NOTIF_HEADER)
        .body(&format!("{message} {data}"))
        .timeout(2000)
        //.hint(Hint::Resident(true))
        .hint(Hint::Category("device".into()));

    notif
}

fn do_notification<T>(
    message: &str,
    data: &T,
) -> Result<NotificationHandle>
where
    T: Display,
{
    Ok(base_notification(message, data).show()?)
}

fn ac_power_notification(
    message: &str,
    on: &bool,
) -> Result<NotificationHandle> {
    let data = if *on { "plugged".to_string() } else { "unplugged".to_string() };
    Ok(base_notification(message, &data).show()?)
}

fn do_thermal_notif(
    message: &str,
    profile: &Profile,
) -> Result<NotificationHandle> {
    let icon = match profile {
        Profile::Balanced => "asus_notif_yellow",
        Profile::Performance => "asus_notif_red",
        Profile::Quiet => "asus_notif_green",
    };
    let profile: &str = (*profile).into();
    let mut notif = base_notification(message, &profile.to_uppercase());
    Ok(notif.icon(icon).show()?)
}
