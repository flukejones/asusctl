use crate::systemd::{do_systemd_unit_action, SystemdUnitAction};
use crate::{config::Config, error::RogError, GetSupported};
use crate::{task_watch_item, CtrlTask};
use async_trait::async_trait;
use log::{error, info, warn};
use rog_platform::power::AsusPower;
use rog_platform::supported::ChargeSupportedFunctions;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use zbus::dbus_interface;
use zbus::export::futures_util::lock::Mutex;
use zbus::Connection;
use zbus::SignalContext;

const ZBUS_PATH: &str = "/org/asuslinux/Power";
const NVIDIA_POWERD: &str = "nvidia-powerd.service";

impl GetSupported for CtrlPower {
    type A = ChargeSupportedFunctions;

    fn get_supported() -> Self::A {
        ChargeSupportedFunctions {
            charge_level_set: if let Ok(power) = AsusPower::new() {
                power.has_charge_control_end_threshold()
            } else {
                false
            },
        }
    }
}

#[derive(Clone)]
pub struct CtrlPower {
    power: AsusPower,
    config: Arc<Mutex<Config>>,
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlPower {
    async fn set_charge_control_end_threshold(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        limit: u8,
    ) -> zbus::fdo::Result<()> {
        if !(20..=100).contains(&limit) {
            return Err(RogError::ChargeLimit(limit))?;
        }
        self.set(limit)
            .map_err(|err| {
                warn!("CtrlCharge: set_limit {}", err);
                err
            })
            .ok();
        Self::notify_charge_control_end_threshold(&ctxt, limit)
            .await
            .ok();
        Ok(())
    }

    fn charge_control_end_threshold(&self) -> u8 {
        loop {
            if let Some(mut config) = self.config.try_lock() {
                let limit = self
                    .power
                    .get_charge_control_end_threshold()
                    .map_err(|err| {
                        warn!("CtrlCharge: get_charge_control_end_threshold {}", err);
                        err
                    })
                    .unwrap_or(100);

                config.read();
                config.bat_charge_limit = limit;
                config.write();

                return config.bat_charge_limit;
            }
        }
    }

    fn mains_online(&self) -> bool {
        if self.power.has_online() {
            if let Ok(v) = self.power.get_online() {
                return v == 1;
            }
        }
        false
    }

    #[dbus_interface(signal)]
    async fn notify_charge_control_end_threshold(
        ctxt: &SignalContext<'_>,
        limit: u8,
    ) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn notify_mains_online(ctxt: &SignalContext<'_>, on: bool) -> zbus::Result<()>;
}

#[async_trait]
impl crate::ZbusRun for CtrlPower {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, ZBUS_PATH, server).await;
    }
}

#[async_trait]
impl crate::Reloadable for CtrlPower {
    async fn reload(&mut self) -> Result<(), RogError> {
        if let Some(mut config) = self.config.try_lock() {
            config.read();
            self.set(config.bat_charge_limit)?;
        }
        Ok(())
    }
}

impl CtrlPower {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        Ok(CtrlPower {
            power: AsusPower::new()?,
            config,
        })
    }

    pub(super) fn set(&self, limit: u8) -> Result<(), RogError> {
        if !(20..=100).contains(&limit) {
            return Err(RogError::ChargeLimit(limit));
        }

        self.power.set_charge_control_end_threshold(limit)?;

        info!("Battery charge limit: {}", limit);

        if let Some(mut config) = self.config.try_lock() {
            config.read();
            config.bat_charge_limit = limit;
            config.write();
        }

        Ok(())
    }

    task_watch_item!(charge_control_end_threshold power);
}

#[async_trait]
impl CtrlTask for CtrlPower {
    fn zbus_path() -> &'static str {
        ZBUS_PATH
    }

    async fn create_tasks(&self, signal_ctxt: SignalContext<'static>) -> Result<(), RogError> {
        let power1 = self.clone();
        let power2 = self.clone();
        self.create_sys_event_tasks(
            move || async {},
            move || {
                let power1 = power1.clone();
                async move {
                    info!("CtrlCharge reloading charge limit");
                    let lock = power1.config.lock().await;
                    power1
                        .set(lock.bat_charge_limit)
                        .map_err(|err| {
                            warn!("CtrlCharge: set_limit {}", err);
                            err
                        })
                        .ok();

                    if let Ok(value) = power1.power.get_online() {
                        let action = if value == 1 {
                            SystemdUnitAction::Restart
                        } else {
                            SystemdUnitAction::Stop
                        };
                        if do_systemd_unit_action(action, NVIDIA_POWERD).is_ok() {
                            info!("CtrlPower task: did {action:?} on {NVIDIA_POWERD}");
                        }
                    }
                }
            },
            move || async {},
            move || {
                let power2 = power2.clone();
                async move {
                    info!("CtrlCharge reloading charge limit");
                    let lock = power2.config.lock().await;
                    power2
                        .set(lock.bat_charge_limit)
                        .map_err(|err| {
                            warn!("CtrlCharge: set_limit {}", err);
                            err
                        })
                        .ok();

                    if let Ok(value) = power2.power.get_online() {
                        let action = if value == 1 {
                            SystemdUnitAction::Restart
                        } else {
                            SystemdUnitAction::Stop
                        };
                        if do_systemd_unit_action(action, NVIDIA_POWERD).is_ok() {
                            info!("CtrlPower task: did {action:?} on {NVIDIA_POWERD}");
                        }
                    }
                }
            },
        )
        .await;

        let config = self.config.clone();
        self.watch_charge_control_end_threshold(signal_ctxt.clone())
            .await?;

        let ctrl = self.clone();
        tokio::spawn(async move {
            let mut online = 10;
            loop {
                if let Ok(value) = ctrl.power.get_online() {
                    if online != value {
                        online = value;
                        let action = if value == 1 {
                            SystemdUnitAction::Restart
                        } else {
                            SystemdUnitAction::Stop
                        };
                        if do_systemd_unit_action(action, NVIDIA_POWERD).is_ok() {
                            info!("CtrlPower task: did {action:?} on {NVIDIA_POWERD}");
                        }

                        Self::notify_mains_online(&signal_ctxt, value == 1)
                            .await
                            .unwrap();

                        let mut config = config.lock().await;
                        config.read();
                        let mut prog: Vec<&str> = Vec::new();
                        if value == 1 {
                            // AC ONLINE
                            prog = config.ac_command.split_whitespace().collect();
                        } else if value == 0 {
                            // BATTERY
                            prog = config.bat_command.split_whitespace().collect();
                        }

                        if prog.len() > 1 {
                            let mut cmd = Command::new(prog[0]);
                            for arg in prog.iter().skip(1) {
                                cmd.arg(*arg);
                            }
                            if let Err(e) = cmd.spawn() {
                                if value == 1 {
                                    error!("AC power command error: {e}");
                                } else {
                                    error!("Battery power command error: {e}");
                                }
                            }
                        }
                    }
                }
                // The inotify doesn't pick up events when the kernel changes internal value
                // so we need to watch it with a thread and sleep unfortunately
                sleep(Duration::from_secs(1)).await;
            }
        });

        Ok(())
    }
}
