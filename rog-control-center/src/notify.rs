//! `update_and_notify` is responsible for both notifications *and* updating
//! stored statuses about the system state. This is done through either direct,
//! intoify, zbus notifications or similar methods.
//!
//! This module very much functions like a stand-alone app on its own thread.

use std::fmt::Display;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{error, info, warn};
use notify_rust::{Hint, Notification, NotificationHandle, Urgency};
use rog_dbus::zbus_platform::PlatformProxy;
use rog_platform::platform::GpuMode;
use rog_platform::power::AsusPower;
use serde::{Deserialize, Serialize};
use supergfxctl::actions::UserActionRequired as GfxUserAction;
use supergfxctl::pci_device::{GfxMode, GfxPower};
use supergfxctl::zbus_proxy::DaemonProxy as SuperProxy;
use tokio::time::sleep;
use zbus::export::futures_util::StreamExt;

use crate::config::Config;
use crate::error::Result;

const NOTIF_HEADER: &str = "ROG Control";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EnabledNotifications {
    pub enabled: bool,
    pub receive_notify_gfx: bool,
    pub receive_notify_gfx_status: bool,
}

impl Default for EnabledNotifications {
    fn default() -> Self {
        Self {
            enabled: true,
            receive_notify_gfx: true,
            receive_notify_gfx_status: true,
        }
    }
}

pub fn start_notifications(config: Arc<Mutex<Config>>) -> Result<()> {
    // Setup the AC/BAT commands that will run on power status change
    let config_copy = config.clone();
    tokio::task::spawn_blocking(move || {
        let power = AsusPower::new()
            .map_err(|e| {
                error!("AsusPower: {e}");
                e
            })
            .unwrap();

        let mut last_state = power.get_online().unwrap_or_default();
        loop {
            if let Ok(p) = power.get_online() {
                let mut ac = String::new();
                let mut bat = String::new();
                if let Ok(config) = config_copy.lock() {
                    ac.clone_from(&config.ac_command);
                    bat.clone_from(&config.bat_command);
                }

                if p == 0 && p != last_state {
                    let prog: Vec<&str> = bat.split_whitespace().collect();
                    if prog.len() > 1 {
                        let mut cmd = Command::new(prog[0]);

                        for arg in prog.iter().skip(1) {
                            cmd.arg(*arg);
                        }
                        cmd.spawn()
                            .map_err(|e| error!("AC command error: {e:?}"))
                            .ok();
                    }
                } else if p != last_state {
                    let prog: Vec<&str> = ac.split_whitespace().collect();
                    if prog.len() > 1 {
                        let mut cmd = Command::new(prog[0]);

                        for arg in prog.iter().skip(1) {
                            cmd.arg(*arg);
                        }
                        cmd.spawn()
                            .map_err(|e| error!("AC command error: {e:?}"))
                            .ok();
                    }
                }
                last_state = p;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    // GPU MUX Mode notif
    let enabled_notifications_copy = config.clone();
    tokio::spawn(async move {
        let conn = zbus::Connection::system()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_gpu_mux_mode: {e}");
                e
            })
            .unwrap();
        let proxy = PlatformProxy::new(&conn)
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_gpu_mux_mode: {e}");
                e
            })
            .unwrap();

        let mut actual_mux_mode = GpuMode::Error;
        if let Ok(mode) = proxy.gpu_mux_mode().await {
            actual_mux_mode = GpuMode::from(mode);
        }

        info!("Started zbus signal thread: receive_notify_gpu_mux_mode");
        while let Some(e) = proxy.receive_gpu_mux_mode_changed().await.next().await {
            if let Ok(config) = enabled_notifications_copy.lock() {
                if !config.notifications.enabled || !config.notifications.receive_notify_gfx {
                    continue;
                }
            }
            if let Ok(out) = e.get().await {
                let mode = GpuMode::from(out);
                if mode == actual_mux_mode {
                    continue;
                }
                do_mux_notification("Reboot required. BIOS GPU MUX mode set to", &mode).ok();
            }
        }
    });

    use supergfxctl::pci_device::Device;
    let dev = Device::find().unwrap_or_default();
    let mut found_dgpu = false; // just for logging
    for dev in dev {
        if dev.is_dgpu() {
            let enabled_notifications_copy = config.clone();
            // Plain old thread is perfectly fine since most of this is potentially blocking
            tokio::spawn(async move {
                let mut last_status = GfxPower::Unknown;
                loop {
                    if let Ok(status) = dev.get_runtime_status() {
                        if status != GfxPower::Unknown && status != last_status {
                            if let Ok(config) = enabled_notifications_copy.lock() {
                                if !config.notifications.receive_notify_gfx_status
                                    || !config.notifications.enabled
                                {
                                    continue;
                                }
                            }
                            // Required check because status cycles through
                            // active/unknown/suspended
                            do_gpu_status_notif("dGPU status changed:", &status).ok();
                        }
                        last_status = status;
                    }
                    sleep(Duration::from_millis(500)).await;
                }
            });
            found_dgpu = true;
            break;
        }
    }
    if !found_dgpu {
        warn!("Did not find a dGPU on this system, dGPU status won't be avilable");
    }

    // GPU Mode change/action notif
    tokio::spawn(async move {
        let conn = zbus::Connection::system()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_action: {e}");
                e
            })
            .unwrap();
        let proxy = SuperProxy::builder(&conn)
            .build()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_action: {e}");
                e
            })
            .unwrap();

        if proxy.mode().await.is_err() {
            info!("supergfxd not running or not responding");
            return;
        }

        if let Ok(mut p) = proxy.receive_notify_action().await {
            info!("Started zbus signal thread: receive_notify_action");
            while let Some(e) = p.next().await {
                if let Ok(out) = e.args() {
                    let action = out.action();
                    let mode = convert_gfx_mode(proxy.mode().await.unwrap_or_default());
                    match action {
                        supergfxctl::actions::UserActionRequired::Reboot => {
                            do_mux_notification("Graphics mode change requires reboot", &mode)
                        }
                        _ => do_gfx_action_notif(<&str>::from(action), *action, mode),
                    }
                    .map_err(|e| {
                        error!("zbus signal: do_gfx_action_notif: {e}");
                        e
                    })
                    .ok();
                }
            }
        };
    });

    Ok(())
}

fn convert_gfx_mode(gfx: GfxMode) -> GpuMode {
    match gfx {
        GfxMode::Hybrid => GpuMode::Optimus,
        GfxMode::Integrated => GpuMode::Integrated,
        GfxMode::NvidiaNoModeset => GpuMode::Optimus,
        GfxMode::Vfio => GpuMode::Vfio,
        GfxMode::AsusEgpu => GpuMode::Egpu,
        GfxMode::AsusMuxDgpu => GpuMode::Ultimate,
        GfxMode::None => GpuMode::Error,
    }
}

fn base_notification<T>(message: &str, data: &T) -> Notification
where
    T: Display,
{
    let mut notif = Notification::new();

    notif
        .summary(NOTIF_HEADER)
        .body(&format!("{message} {data}"))
        .timeout(-1)
        //.hint(Hint::Resident(true))
        .hint(Hint::Category("device".into()));

    notif
}

fn do_gpu_status_notif(message: &str, data: &GfxPower) -> Result<NotificationHandle> {
    // eww
    let mut notif = base_notification(message, &<&str>::from(data).to_owned());
    let icon = match data {
        GfxPower::Suspended => "asus_notif_blue",
        GfxPower::Off => "asus_notif_green",
        GfxPower::AsusDisabled => "asus_notif_white",
        GfxPower::AsusMuxDiscreet | GfxPower::Active => "asus_notif_red",
        GfxPower::Unknown => "gpu-integrated",
    };
    notif.icon(icon);
    Ok(Notification::show(&notif)?)
}

fn do_gfx_action_notif(message: &str, action: GfxUserAction, mode: GpuMode) -> Result<()> {
    if matches!(action, GfxUserAction::Reboot) {
        do_mux_notification("Graphics mode change requires reboot", &mode).ok();
        return Ok(());
    }

    let mut notif = Notification::new();
    notif
        .summary(NOTIF_HEADER)
        .body(&format!("Changing to {mode}. {message}"))
        .timeout(2000)
        //.hint(Hint::Resident(true))
        .hint(Hint::Category("device".into()))
        .urgency(Urgency::Critical)
        .timeout(-1)
        .icon("dialog-warning")
        .hint(Hint::Transient(true));

    if matches!(action, GfxUserAction::Logout) {
        notif.action("gfx-mode-session-action", "Logout");
        let handle = notif.show()?;
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            if desktop.to_lowercase() == "gnome" {
                handle.wait_for_action(|id| {
                    if id == "gfx-mode-session-action" {
                        let mut cmd = Command::new("gnome-session-quit");
                        cmd.spawn().ok();
                    } else if id == "__closed" {
                        // TODO: cancel the switching
                    }
                });
            } else if desktop.to_lowercase() == "kde" {
                handle.wait_for_action(|id| {
                    if id == "gfx-mode-session-action" {
                        let mut cmd = Command::new("qdbus");
                        cmd.args(["org.kde.ksmserver", "/KSMServer", "logout", "1", "0", "0"]);
                        cmd.spawn().ok();
                    } else if id == "__closed" {
                        // TODO: cancel the switching
                    }
                });
            } else {
                // todo: handle alternatives
            }
        }
    } else {
        notif.show()?;
    }
    Ok(())
}

/// Actual `GpuMode` unused as data is never correct until switched by reboot
fn do_mux_notification(message: &str, m: &GpuMode) -> Result<()> {
    let mut notif = base_notification(message, &m.to_string());
    notif
        .action("gfx-mode-session-action", "Reboot")
        .urgency(Urgency::Critical)
        .icon("system-reboot-symbolic")
        .hint(Hint::Transient(true));
    let handle = notif.show()?;

    std::thread::spawn(|| {
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            if desktop.to_lowercase() == "gnome" {
                handle.wait_for_action(|id| {
                    if id == "gfx-mode-session-action" {
                        let mut cmd = Command::new("gnome-session-quit");
                        cmd.arg("--reboot");
                        cmd.spawn().ok();
                    } else if id == "__closed" {
                        // TODO: cancel the switching
                    }
                });
            } else if desktop.to_lowercase() == "kde" {
                handle.wait_for_action(|id| {
                    if id == "gfx-mode-session-action" {
                        let mut cmd = Command::new("qdbus");
                        cmd.args(["org.kde.ksmserver", "/KSMServer", "logout", "1", "1", "0"]);
                        cmd.spawn().ok();
                    } else if id == "__closed" {
                        // TODO: cancel the switching
                    }
                });
            }
        }
    });
    Ok(())
}
