use crate::{config::Config, error::Result, system_state::SystemState};
use log::{error, info, trace};
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
    sync::{Arc, Mutex},
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
        $last_notif:ident,
        $notif_enabled:ident,
        $page_states:ident,
        ($($args: tt)*),
        ($($out_arg:tt)+),
        $msg:literal,
        $notifier:ident) => {

        let last_notif = $last_notif.clone();
        let notifs_enabled1 = $notif_enabled.clone();
        let page_states1 = $page_states.clone();

        tokio::spawn(async move {
                let conn = zbus::Connection::system().await.map_err(|e| {
                        log::error!("zbus signal: {}: {e}", stringify!($signal));
                        e
                    }).unwrap();
                let proxy = $proxy::new(&conn).await.map_err(|e| {
                        log::error!("zbus signal: {}: {e}", stringify!($signal));
                        e
                    }).unwrap();
                if let Ok(mut p) = proxy.$signal().await {
                    info!("Started zbus signal thread: {}", stringify!($signal));
                    while let Some(e) = p.next().await {
                        if let Ok(out) = e.args() {
                            if let Ok(config) = notifs_enabled1.lock() {
                                if config.all_enabled && config.$signal {
                                    if let Ok(ref mut lock) = last_notif.lock() {
                                        trace!("zbus signal {} locked last_notif", stringify!($signal));
                                        notify!($notifier($msg, &out.$($out_arg)+()), lock);
                                    }
                                }
                            }
                            if let Ok(mut lock) = page_states1.lock() {
                                lock.$($args)+ = *out.$($out_arg)+();
                                lock.set_notified();
                            }
                        }
                    }
                };
            });
    };
}

type SharedHandle = Arc<Mutex<Option<NotificationHandle>>>;

pub fn start_notifications(
    page_states: Arc<Mutex<SystemState>>,
    enabled_notifications: Arc<Mutex<EnabledNotifications>>,
) -> Result<()> {
    let last_notification: SharedHandle = Arc::new(Mutex::new(None));

    // BIOS notif
    recv_notif!(
        RogBiosProxy,
        receive_notify_post_boot_sound,
        last_notification,
        enabled_notifications,
        page_states,
        (bios.post_sound),
        (on),
        "BIOS Post sound",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_panel_od,
        last_notification,
        enabled_notifications,
        page_states,
        (bios.panel_overdrive),
        (overdrive),
        "Panel Overdrive enabled:",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_dgpu_disable,
        last_notification,
        enabled_notifications,
        page_states,
        (bios.dgpu_disable),
        (disable),
        "BIOS dGPU disabled",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_egpu_enable,
        last_notification,
        enabled_notifications,
        page_states,
        (bios.egpu_enable),
        (enable),
        "BIOS eGPU enabled",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_gpu_mux_mode,
        last_notification,
        enabled_notifications,
        page_states,
        (bios.dedicated_gfx),
        (mode),
        "Reboot required. BIOS GPU MUX mode set to",
        do_mux_notification
    );

    // Charge notif
    recv_notif!(
        PowerProxy,
        receive_notify_charge_control_end_threshold,
        last_notification,
        enabled_notifications,
        page_states,
        (power_state.charge_limit),
        (limit),
        "Battery charge limit changed to",
        do_notification
    );

    recv_notif!(
        PowerProxy,
        receive_notify_mains_online,
        last_notification,
        enabled_notifications,
        page_states,
        (power_state.ac_power),
        (on),
        "AC Power power is",
        ac_power_notification
    );

    // Profile notif
    recv_notif!(
        ProfileProxy,
        receive_notify_profile,
        last_notification,
        enabled_notifications,
        page_states,
        (profiles.current),
        (profile),
        "Profile changed to",
        do_thermal_notif
    );
    // notify!(do_thermal_notif(&out.profile), lock);

    // LED notif
    recv_notif!(
        LedProxy,
        receive_notify_led,
        last_notification,
        enabled_notifications,
        page_states,
        (aura.current_mode),
        (data.mode),
        "Keyboard LED mode changed to",
        do_notification
    );

    let page_states1 = page_states.clone();
    tokio::spawn(async move {
        let conn = zbus::Connection::system()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_power_states: {e}");
                e
            })
            .unwrap();
        let proxy = AnimeProxy::new(&conn)
            .await
            .map_err(|e| {
                error!("zbus signal: receive_power_states: {e}");
                e
            })
            .unwrap();
        if let Ok(p) = proxy.receive_power_states().await {
            info!("Started zbus signal thread: receive_power_states");
            p.for_each(|_| {
                if let Ok(_lock) = page_states1.lock() {
                    // TODO: lock.anime.
                }
                future::ready(())
            })
            .await;
        };
    });

    recv_notif!(
        SuperProxy,
        receive_notify_gfx,
        last_notification,
        enabled_notifications,
        page_states,
        (gfx_state.mode),
        (mode),
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
        let conn = zbus::Connection::system()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_action: {e}");
                e
            })
            .unwrap();
        let proxy = SuperProxy::new(&conn)
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_action: {e}");
                e
            })
            .unwrap();
        if let Ok(mut p) = proxy.receive_notify_action().await {
            info!("Started zbus signal thread: receive_notify_action");
            while let Some(e) = p.next().await {
                if let Ok(out) = e.args() {
                    let action = out.action();
                    do_gfx_action_notif("Gfx mode change requires", &format!("{action:?}",))
                        .map_err(|e| {
                            error!("zbus signal: do_gfx_action_notif: {e}");
                            e
                        })
                        .unwrap();
                }
            }
        };
    });

    let notifs_enabled1 = enabled_notifications;
    let last_notif = last_notification;
    tokio::spawn(async move {
        let conn = zbus::Connection::system()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_gfx_status: {e}");
                e
            })
            .unwrap();
        let proxy = SuperProxy::new(&conn)
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_gfx_status: {e}");
                e
            })
            .unwrap();
        if let Ok(mut p) = proxy.receive_notify_gfx_status().await {
            info!("Started zbus signal thread: receive_notify_gfx_status");
            while let Some(e) = p.next().await {
                if let Ok(out) = e.args() {
                    let status = out.status;
                    if status != GfxPower::Unknown {
                        if let Ok(config) = notifs_enabled1.lock() {
                            if config.all_enabled && config.receive_notify_gfx_status {
                                // Required check because status cycles through active/unknown/suspended
                                if let Ok(ref mut lock) = last_notif.lock() {
                                    notify!(
                                        do_gpu_status_notif("dGPU status changed:", &status),
                                        lock
                                    );
                                }
                            }
                        }
                        if let Ok(mut lock) = page_states.lock() {
                            lock.gfx_state.power_status = status;
                            lock.set_notified();
                        }
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
