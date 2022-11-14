use crate::{config::Config, error::Result};
use notify_rust::{Hint, Notification, NotificationHandle, Urgency};
use rog_dbus::{
    zbus_anime::AnimeProxy, zbus_led::LedProxy, zbus_platform::RogBiosProxy,
    zbus_power::PowerProxy, zbus_profile::ProfileProxy,
};
use rog_platform::platform::GpuMode;
use rog_profiles::Profile;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use supergfxctl::{pci_device::GfxPower, zbus_proxy::DaemonProxy as SuperProxy};
use zbus::export::futures_util::{future, StreamExt};

const NOTIF_HEADER: &str = "ROG Control";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EnabledNotifications {
    pub receive_notify_post_boot_sound: bool,
    pub receive_notify_panel_od: bool,
    pub receive_notify_dgpu_disable: bool,
    pub receive_notify_egpu_enable: bool,
    pub receive_notify_gpu_mux_mode: bool,
    pub receive_notify_charge_control_end_threshold: bool,
    pub receive_notify_mains_online: bool,
    pub receive_notify_profile: bool,
    pub receive_notify_led: bool,
    /// Anime
    pub receive_power_states: bool,
    pub receive_notify_gfx: bool,
    pub receive_notify_gfx_status: bool,
    pub all_enabled: bool,
}

impl Default for EnabledNotifications {
    fn default() -> Self {
        Self {
            receive_notify_post_boot_sound: false,
            receive_notify_panel_od: true,
            receive_notify_dgpu_disable: true,
            receive_notify_egpu_enable: true,
            receive_notify_gpu_mux_mode: true,
            receive_notify_charge_control_end_threshold: true,
            receive_notify_mains_online: false,
            receive_notify_profile: true,
            receive_notify_led: false,
            receive_power_states: false,
            receive_notify_gfx: false,
            receive_notify_gfx_status: false,
            all_enabled: false,
        }
    }
}

impl EnabledNotifications {
    pub fn tokio_mutex(config: &Config) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(config.enabled_notifications.clone()))
    }
}

/// Intended as a help to determine if daemon controllers notified state
#[derive(Debug, Default, Clone)]
pub struct WasNotified {
    pub charge: Arc<AtomicBool>,
    pub bios: Arc<AtomicBool>,
    pub aura: Arc<AtomicBool>,
    pub anime: Arc<AtomicBool>,
    pub profiles: Arc<AtomicBool>,
    pub fans: Arc<AtomicBool>,
    pub gfx: Arc<AtomicBool>,
}

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

// TODO: drop the macro and use generics plus closure
macro_rules! recv_notif {
    ($proxy:ident,
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
        tokio::spawn(async move {
                let conn = zbus::Connection::system().await.unwrap();
                let proxy = $proxy::new(&conn).await.unwrap();
                if let Ok(mut p) = proxy.$signal().await {
                    while let Some(e) = p.next().await {
                        if let Ok(out) = e.args() {
                            if let Ok(config) = notifs_enabled1.lock() {
                                if config.all_enabled && config.$signal {
                                    if let Ok(ref mut lock) = last_notif.try_lock() {
                                        notify!($notifier($msg, &out$(.$out_arg)+()), lock);
                                    }
                                }
                            }
                            notified.store(true, Ordering::SeqCst);
                        }
                    }
                };
            });
    };
}

type SharedHandle = Arc<Mutex<Option<NotificationHandle>>>;

pub fn start_notifications(
    was_notified: WasNotified,
    enabled_notifications: Arc<Mutex<EnabledNotifications>>,
) -> Result<()> {
    let last_notification: SharedHandle = Arc::new(Mutex::new(None));

    let WasNotified {
        bios: bios_notified,
        charge: charge_notified,
        profiles: profiles_notified,
        aura: aura_notified,
        anime: anime_notified,
        gfx: gfx_notified,
        ..
    } = was_notified;

    // BIOS notif
    recv_notif!(
        RogBiosProxy,
        receive_notify_post_boot_sound,
        bios_notified,
        last_notification,
        enabled_notifications,
        [on],
        "BIOS Post sound",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_panel_od,
        bios_notified,
        last_notification,
        enabled_notifications,
        [overdrive],
        "Panel Overdrive enabled:",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_dgpu_disable,
        bios_notified,
        last_notification,
        enabled_notifications,
        [disable],
        "BIOS dGPU disabled",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_egpu_enable,
        bios_notified,
        last_notification,
        enabled_notifications,
        [enable],
        "BIOS eGPU enabled",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_gpu_mux_mode,
        bios_notified,
        last_notification,
        enabled_notifications,
        [mode],
        "Reboot required. BIOS GPU MUX mode set to",
        do_mux_notification
    );

    // Charge notif
    recv_notif!(
        PowerProxy,
        receive_notify_charge_control_end_threshold,
        charge_notified,
        last_notification,
        enabled_notifications,
        [limit],
        "Battery charge limit changed to",
        do_notification
    );

    recv_notif!(
        PowerProxy,
        receive_notify_mains_online,
        bios_notified,
        last_notification,
        enabled_notifications,
        [on],
        "AC Power power is",
        ac_power_notification
    );

    // Profile notif
    recv_notif!(
        ProfileProxy,
        receive_notify_profile,
        profiles_notified,
        last_notification,
        enabled_notifications,
        [profile],
        "Profile changed to",
        do_thermal_notif
    );
    // notify!(do_thermal_notif(&out.profile), lock);

    // LED notif
    recv_notif!(
        LedProxy,
        receive_notify_led,
        aura_notified,
        last_notification,
        enabled_notifications,
        [data mode_name],
        "Keyboard LED mode changed to",
        do_notification
    );

    tokio::spawn(async move {
        let conn = zbus::Connection::system().await.unwrap();
        let proxy = LedProxy::new(&conn).await.unwrap();
        if let Ok(p) = proxy.receive_all_signals().await {
            p.for_each(|_| {
                aura_notified.store(true, Ordering::SeqCst);
                future::ready(())
            })
            .await;
        };
    });

    tokio::spawn(async move {
        let conn = zbus::Connection::system().await.unwrap();
        let proxy = AnimeProxy::new(&conn).await.unwrap();
        if let Ok(p) = proxy.receive_power_states().await {
            p.for_each(|_| {
                anime_notified.store(true, Ordering::SeqCst);
                future::ready(())
            })
            .await;
        };
    });

    recv_notif!(
        SuperProxy,
        receive_notify_gfx,
        bios_notified,
        last_notification,
        enabled_notifications,
        [mode],
        "Gfx mode changed to",
        do_notification
    );

    // recv_notif!(
    //     SuperProxy,
    //     receive_notify_action,
    //     bios_notified,
    //     last_gfx_action_notif,
    //     enabled_notifications,
    //     [action],
    //     "Gfx mode change requires",
    //     do_gfx_action_notif
    // );

    tokio::spawn(async move {
        let conn = zbus::Connection::system().await.unwrap();
        let proxy = SuperProxy::new(&conn).await.unwrap();
        if let Ok(mut p) = proxy.receive_notify_action().await {
            while let Some(e) = p.next().await {
                if let Ok(out) = e.args() {
                    let action = out.action();
                    do_gfx_action_notif("Gfx mode change requires", &format!("{action:?}",))
                        .unwrap();
                    bios_notified.store(true, Ordering::SeqCst);
                }
            }
        };
    });

    let notifs_enabled1 = enabled_notifications;
    let last_notif = last_notification;
    tokio::spawn(async move {
        let conn = zbus::Connection::system().await.unwrap();
        let proxy = SuperProxy::new(&conn).await.unwrap();
        if let Ok(mut p) = proxy.receive_notify_gfx_status().await {
            while let Some(e) = p.next().await {
                if let Ok(out) = e.args() {
                    let status = out.status();
                    if *status != GfxPower::Unknown {
                        if let Ok(config) = notifs_enabled1.lock() {
                            if config.all_enabled && config.receive_notify_gfx_status {
                                // Required check because status cycles through active/unknown/suspended
                                if let Ok(ref mut lock) = last_notif.try_lock() {
                                    notify!(
                                        do_gpu_status_notif("dGPU status changed:", status),
                                        lock
                                    );
                                }
                            }
                        }
                        gfx_notified.store(true, Ordering::SeqCst);
                    }
                }
            }
        };
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

fn do_notification<T>(message: &str, data: &T) -> Result<NotificationHandle>
where
    T: Display,
{
    Ok(base_notification(message, data).show()?)
}

fn ac_power_notification(message: &str, on: &bool) -> Result<NotificationHandle> {
    let data = if *on {
        "plugged".to_string()
    } else {
        "unplugged".to_string()
    };
    Ok(base_notification(message, &data).show()?)
}

fn do_thermal_notif(message: &str, profile: &Profile) -> Result<NotificationHandle> {
    let icon = match profile {
        Profile::Balanced => "asus_notif_yellow",
        Profile::Performance => "asus_notif_red",
        Profile::Quiet => "asus_notif_green",
    };
    let profile: &str = (*profile).into();
    let mut notif = base_notification(message, &profile.to_uppercase());
    Ok(notif.icon(icon).show()?)
}

fn do_gpu_status_notif(message: &str, data: &GfxPower) -> Result<NotificationHandle> {
    // eww
    let mut notif = base_notification(message, &<&str>::from(data).to_string());
    let icon = match data {
        GfxPower::Active => "asus_notif_red",
        GfxPower::Suspended => "asus_notif_blue",
        GfxPower::Off => "asus_notif_green",
        GfxPower::AsusDisabled => "asus_notif_white",
        GfxPower::AsusMuxDiscreet => "asus_notif_red",
        GfxPower::Unknown => "gpu-integrated",
    };
    notif.icon(icon);
    Ok(Notification::show(&notif)?)
}

fn do_gfx_action_notif<T>(message: &str, data: &T) -> Result<()>
where
    T: Display,
{
    let mut notif = base_notification(message, data);
    notif.action("gnome-session-quit", "Logout");
    notif.urgency(Urgency::Critical);
    notif.timeout(3000);
    notif.icon("dialog-warning");
    notif.hint(Hint::Transient(true));
    let handle = notif.show()?;
    handle.wait_for_action(|id| {
        if id == "gnome-session-quit" {
            let mut cmd = Command::new("gnome-session-quit");
            cmd.spawn().ok();
        } else if id == "__closed" {
            // TODO: cancel the switching
        }
    });
    Ok(())
}

/// Actual GpuMode unused as data is never correct until switched by reboot
fn do_mux_notification(message: &str, _: &GpuMode) -> Result<NotificationHandle> {
    let mut notif = base_notification(message, &"");
    notif.urgency(Urgency::Critical);
    notif.icon("system-reboot-symbolic");
    notif.hint(Hint::Transient(true));
    Ok(notif.show()?)
}
