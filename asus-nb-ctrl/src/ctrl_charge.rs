use crate::config::Config;
use log::{error, info, warn};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

static BAT_CHARGE_PATH: &str = "/sys/class/power_supply/BAT0/charge_control_end_threshold";

pub struct CtrlCharge {
    path: &'static str,
}

use ::dbus::{nonblock::SyncConnection, tree::Signal};
use async_trait::async_trait;

#[async_trait]
impl crate::Controller for CtrlCharge {
    type A = u8;

    /// Spawns two tasks which continuously check for changes
    fn spawn_task_loop(
        self,
        config: Arc<Mutex<Config>>,
        mut recv: Receiver<Self::A>,
        _: Option<Arc<SyncConnection>>,
        _: Option<Arc<Signal<()>>>,
    ) -> Vec<JoinHandle<()>> {
        vec![tokio::spawn(async move {
            while let Some(n) = recv.recv().await {
                let mut config = config.lock().await;
                self.set_charge_limit(n, &mut config)
                    .unwrap_or_else(|err| warn!("charge_limit: {}", err));
            }
        })]
    }

    async fn reload_from_config(&mut self, config: &mut Config) -> Result<(), Box<dyn Error>> {
        config.read();
        info!("Reloaded battery charge limit");
        self.set_charge_limit(config.bat_charge_limit, config)
    }
}

impl CtrlCharge {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path = CtrlCharge::get_battery_path()?;
        info!("Device has battery charge threshold control");
        Ok(CtrlCharge { path })
    }

    fn get_battery_path() -> Result<&'static str, std::io::Error> {
        if Path::new(BAT_CHARGE_PATH).exists() {
            Ok(BAT_CHARGE_PATH)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Charge control not available",
            ))
        }
    }

    pub(super) fn set_charge_limit(
        &self,
        limit: u8,
        config: &mut Config,
    ) -> Result<(), Box<dyn Error>> {
        if limit < 20 || limit > 100 {
            warn!(
                "Unable to set battery charge limit, must be between 20-100: requested {}",
                limit
            );
        }

        let mut file = OpenOptions::new()
            .write(true)
            .open(self.path)
            .map_err(|err| {
                warn!("Failed to open battery charge limit path: {}", err);
                err
            })?;
        file.write_all(limit.to_string().as_bytes())
            .unwrap_or_else(|err| error!("Could not write to {}, {}", BAT_CHARGE_PATH, err));
        info!("Battery charge limit: {}", limit);

        config.read();
        config.bat_charge_limit = limit;
        config.write();

        Ok(())
    }
}
