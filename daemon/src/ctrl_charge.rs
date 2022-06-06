use crate::CtrlTask;
use crate::{config::Config, error::RogError, GetSupported};
use async_trait::async_trait;
use log::{info, warn};
use logind_zbus::manager::ManagerProxy;
use rog_supported::ChargeSupportedFunctions;
use smol::stream::StreamExt;
use smol::Executor;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::dbus_interface;
use zbus::Connection;
use zbus::SignalContext;

static BAT_CHARGE_PATH0: &str = "/sys/class/power_supply/BAT0/charge_control_end_threshold";
static BAT_CHARGE_PATH1: &str = "/sys/class/power_supply/BAT1/charge_control_end_threshold";
static BAT_CHARGE_PATH2: &str = "/sys/class/power_supply/BAT2/charge_control_end_threshold";

impl GetSupported for CtrlCharge {
    type A = ChargeSupportedFunctions;

    fn get_supported() -> Self::A {
        ChargeSupportedFunctions {
            charge_level_set: CtrlCharge::get_battery_path().is_ok(),
        }
    }
}

pub struct CtrlCharge {
    config: Arc<Mutex<Config>>,
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlCharge {
    async fn set_limit(
        &mut self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        limit: u8,
    ) -> zbus::fdo::Result<()> {
        if !(20..=100).contains(&limit) {
            return Err(RogError::ChargeLimit(limit))?;
        }
        if let Ok(mut config) = self.config.try_lock() {
            Self::set(limit, &mut config)
                .map_err(|err| {
                    warn!("CtrlCharge: set_limit {}", err);
                    err
                })
                .ok();
        }
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
impl crate::ZbusAdd for CtrlCharge {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, "/org/asuslinux/Charge", server).await;
    }
}

impl crate::Reloadable for CtrlCharge {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.try_lock() {
            config.read();
            Self::set(config.bat_charge_limit, &mut config)?;
        }
        Ok(())
    }
}

impl CtrlCharge {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        CtrlCharge::get_battery_path()?;
        Ok(CtrlCharge { config })
    }

    fn get_battery_path() -> Result<&'static str, RogError> {
        if Path::new(BAT_CHARGE_PATH0).exists() {
            Ok(BAT_CHARGE_PATH0)
        } else if Path::new(BAT_CHARGE_PATH1).exists() {
            Ok(BAT_CHARGE_PATH1)
        } else if Path::new(BAT_CHARGE_PATH2).exists() {
            Ok(BAT_CHARGE_PATH2)
        } else {
            Err(RogError::MissingFunction(
                "Charge control not available, you may require a v5.8.10 series kernel or newer"
                    .into(),
            ))
        }
    }

    pub(super) fn set(limit: u8, config: &mut Config) -> Result<(), RogError> {
        if !(20..=100).contains(&limit) {
            return Err(RogError::ChargeLimit(limit));
        }

        let path = Self::get_battery_path()?;

        let mut file = OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|err| RogError::Path(path.into(), err))?;
        file.write_all(limit.to_string().as_bytes())
            .map_err(|err| RogError::Write(path.into(), err))?;
        info!("Battery charge limit: {}", limit);

        config.read();
        config.bat_charge_limit = limit;
        config.write();

        Ok(())
    }
}

#[async_trait]
impl CtrlTask for CtrlCharge {
    async fn create_tasks(&self, executor: &mut Executor) -> Result<(), RogError> {
        let connection = Connection::system()
            .await
            .expect("CtrlCharge could not create dbus connection");

        let manager = ManagerProxy::new(&connection)
            .await
            .expect("CtrlCharge could not create ManagerProxy");

        let config1 = self.config.clone();
        executor
            .spawn(async move {
                if let Ok(notif) = manager.receive_prepare_for_sleep().await {
                    notif
                        .for_each(|event| {
                            if let Ok(args) = event.args() {
                                // If waking up
                                if !args.start {
                                    info!("CtrlCharge reloading charge limit");
                                    if let Ok(mut lock) = config1.try_lock() {
                                        Self::set(lock.bat_charge_limit, &mut lock)
                                            .map_err(|err| {
                                                warn!("CtrlCharge: set_limit {}", err);
                                                err
                                            })
                                            .ok();
                                    }
                                }
                            }
                        })
                        .await;
                }
            })
            .detach();

        let manager = ManagerProxy::new(&connection)
            .await
            .expect("CtrlCharge could not create ManagerProxy");

        let config = self.config.clone();
        executor
            .spawn(async move {
                if let Ok(notif) = manager.receive_prepare_for_shutdown().await {
                    notif
                        .for_each(|event| {
                            if let Ok(args) = event.args() {
                                // If waking up - intention is to catch hibernation event
                                if !args.start {
                                    info!("CtrlCharge reloading charge limit");
                                    if let Ok(mut lock) = config.clone().try_lock() {
                                        Self::set(lock.bat_charge_limit, &mut lock)
                                            .map_err(|err| {
                                                warn!("CtrlCharge: set_limit {}", err);
                                                err
                                            })
                                            .ok();
                                    }
                                }
                            }
                        })
                        .await;
                }
            })
            .detach();
        Ok(())
    }
}
