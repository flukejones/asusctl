use std::sync::Arc;

use config::ScsiConfig;
use futures_util::lock::{Mutex, MutexGuard};
use rog_scsi::{AuraEffect, Device, Task};

use crate::error::RogError;

pub mod config;
pub mod trait_impls;

#[derive(Clone)]
pub struct ScsiAura {
    device: Arc<Mutex<Device>>,
    config: Arc<Mutex<ScsiConfig>>
}

impl ScsiAura {
    pub fn new(device: Arc<Mutex<Device>>, config: Arc<Mutex<ScsiConfig>>) -> Self {
        Self { device, config }
    }

    pub async fn lock_config(&self) -> MutexGuard<ScsiConfig> {
        self.config.lock().await
    }

    pub async fn write_effect(&self, effect: &AuraEffect) -> Result<(), RogError> {
        let tasks: Vec<Task> = effect.into();
        for task in &tasks {
            self.device.lock().await.perform(task).ok();
        }
        Ok(())
    }

    /// Initialise the device if required. Locks the internal config so be wary
    /// of deadlocks.
    pub async fn do_initialization(&self) -> Result<(), RogError> {
        let config = self.config.lock().await;
        let mode = config.current_mode;
        if let Some(effect) = config.modes.get(&mode) {
            self.write_effect(effect).await?;
        }
        Ok(())
    }
}
