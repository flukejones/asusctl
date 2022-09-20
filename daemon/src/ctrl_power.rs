use crate::CtrlTask;
use crate::{config::Config, error::RogError, GetSupported};
use async_trait::async_trait;
use log::{info, warn};
use rog_platform::power::AsusPower;
use rog_platform::supported::ChargeSupportedFunctions;
use smol::Executor;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::dbus_interface;
use zbus::Connection;
use zbus::SignalContext;

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
    async fn set_limit(
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
        Self::notify_charge(&ctxt, limit).await?;
        Ok(())
    }

    fn limit(&self) -> i8 {
        if let Ok(config) = self.config.try_lock() {
            return config.bat_charge_limit as i8;
        }
        -1
    }

    #[dbus_interface(signal)]
    async fn notify_charge(ctxt: &SignalContext<'_>, limit: u8) -> zbus::Result<()>;
}

#[async_trait]
impl crate::ZbusAdd for CtrlPower {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, "/org/asuslinux/Charge", server).await;
    }
}

impl crate::Reloadable for CtrlPower {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.try_lock() {
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

        if let Ok(mut config) = self.config.try_lock() {
            config.read();
            config.bat_charge_limit = limit;
            config.write();
        }

        Ok(())
    }
}

#[async_trait]
impl CtrlTask for CtrlPower {
    async fn create_tasks<'a>(
        &self,
        executor: &mut Executor<'a>,
        _: SignalContext<'a>,
    ) -> Result<(), RogError> {
        let power1 = self.clone();
        let power2 = self.clone();
        self.create_sys_event_tasks(
            executor,
            move || {},
            move || {
                info!("CtrlCharge reloading charge limit");
                if let Ok(lock) = power1.config.try_lock() {
                    power1
                        .set(lock.bat_charge_limit)
                        .map_err(|err| {
                            warn!("CtrlCharge: set_limit {}", err);
                            err
                        })
                        .ok();
                }
            },
            move || {},
            move || {
                info!("CtrlCharge reloading charge limit");
                if let Ok(lock) = power2.config.try_lock() {
                    power2
                        .set(lock.bat_charge_limit)
                        .map_err(|err| {
                            warn!("CtrlCharge: set_limit {}", err);
                            err
                        })
                        .ok();
                }
            },
        )
        .await;

        Ok(())
    }
}
