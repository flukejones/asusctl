use std::sync::Arc;

use futures_util::lock::Mutex;
use log::{info, warn};
use rog_platform::backlight::{Backlight, BacklightType};
use zbus::fdo::Error as FdoErr;
use zbus::object_server::SignalEmitter;
use zbus::{interface, Connection};

use crate::error::RogError;
use crate::ASUS_ZBUS_PATH;

#[derive(Debug, Clone)]
pub struct CtrlBacklight {
    backlights: Vec<Backlight>,
    sync_all: Arc<Mutex<bool>>,
    gamma: Arc<Mutex<f32>>,
}

impl CtrlBacklight {
    pub fn new() -> Result<Self, RogError> {
        let mut backlights = Vec::new();

        if let Ok(primary) = Backlight::new(BacklightType::Primary) {
            info!("Found primary display backlight");
            backlights.push(primary);
        }

        if let Ok(screenpad) = Backlight::new(BacklightType::Screenpad) {
            info!("Found screenpad backlight");
            backlights.push(screenpad);
        }

        if backlights.is_empty() {
            return Err(RogError::MissingFunction("No backlights found".into()));
        }

        Ok(Self {
            backlights,
            sync_all: Arc::new(Mutex::new(true)),
            gamma: Arc::new(Mutex::new(1.0)),
        })
    }

    fn get_backlight(&self, device_type: &BacklightType) -> Option<&Backlight> {
        self.backlights
            .iter()
            .find(|b| b.device_type() == device_type)
    }

    async fn set_brightness_with_sync(
        &self,
        device_type: &BacklightType,
        level: i32,
    ) -> Result<(), FdoErr> {
        if let Some(backlight) = self.get_backlight(device_type) {
            let max = backlight.get_max_brightness().map_err(|e| {
                warn!("Failed to get max brightness: {}", e);
                FdoErr::Failed(format!("Failed to get max brightness: {}", e))
            })?;

            let scaled = if *device_type == BacklightType::Screenpad {
                // Apply non-linear scaling with the configurable gamma value only for Screenpad
                let gamma = *self.gamma.lock().await;
                let normalized_level = level as f32 / 100.0;
                let gamma_corrected = normalized_level.powf(gamma);
                (gamma_corrected * max as f32) as i32
            } else {
                // Linear scaling for other devices
                level * max / 100
            };

            backlight.set_brightness(scaled).map_err(|e| {
                warn!("Failed to set brightness: {}", e);
                FdoErr::Failed(format!("Failed to set brightness: {}", e))
            })?;

            if *self.sync_all.lock().await {
                for other in self
                    .backlights
                    .iter()
                    .filter(|b| b.device_type() != device_type)
                {
                    if let Ok(other_max) = other.get_max_brightness() {
                        let other_scaled = if other.device_type() == &BacklightType::Screenpad {
                            // Apply gamma only to Screenpad
                            let gamma = *self.gamma.lock().await;
                            let normalized_level = level as f32 / 100.0;
                            let gamma_corrected = normalized_level.powf(gamma);
                            (gamma_corrected * other_max as f32) as i32
                        } else {
                            // Linear scaling for other devices
                            level * other_max / 100
                        };
                        let _ = other.set_brightness(other_scaled);
                    }
                }
            }

            Ok(())
        } else {
            Err(FdoErr::NotSupported(format!(
                "Backlight {:?} not found",
                device_type
            )))
        }
    }

    fn get_brightness_percent(&self, device_type: &BacklightType) -> Result<i32, FdoErr> {
        if let Some(backlight) = self.get_backlight(device_type) {
            let brightness = backlight.get_brightness().map_err(|e| {
                warn!("Failed to get brightness: {}", e);
                FdoErr::Failed(format!("Failed to get brightness: {}", e))
            })?;

            let max = backlight.get_max_brightness().map_err(|e| {
                warn!("Failed to get max brightness: {}", e);
                FdoErr::Failed(format!("Failed to get max brightness: {}", e))
            })?;

            Ok((brightness as u32 * 100 / max as u32) as i32)
        } else {
            Err(FdoErr::NotSupported(format!(
                "Backlight {:?} not found",
                device_type
            )))
        }
    }
}

#[interface(name = "xyz.ljones.Backlight")]
impl CtrlBacklight {
    #[zbus(property)]
    async fn screenpad_sync_with_primary(&self) -> bool {
        *self.sync_all.lock().await
    }

    #[zbus(property)]
    async fn set_screenpad_sync_with_primary(&self, sync: bool) -> Result<(), zbus::Error> {
        *self.sync_all.lock().await = sync;
        Ok(())
    }

    #[zbus(property)]
    async fn screenpad_gamma(&self) -> String {
        (*self.gamma.lock().await).to_string()
    }

    #[zbus(property)]
    async fn set_screenpad_gamma(&self, value: &str) -> Result<(), zbus::Error> {
        let gamma: f32 = value
            .parse()
            .map_err(|_| FdoErr::Failed("Invalid gamma value, must be a valid number".into()))?;

        if gamma < 0.1 {
            return Err(FdoErr::Failed("Gamma value must be greater than 0".into()).into());
        }
        if gamma > 2.0 {
            return Err(FdoErr::Failed("Gamma value must be 2.0 or less".into()).into());
        }
        *self.gamma.lock().await = gamma;
        Ok(())
    }

    #[zbus(property)]
    async fn primary_brightness(&self) -> Result<i32, FdoErr> {
        self.get_brightness_percent(&BacklightType::Primary)
    }

    #[zbus(property)]
    async fn set_primary_brightness(
        &self,
        #[zbus(signal_context)] ctxt: SignalEmitter<'_>,
        level: i32,
    ) -> Result<(), zbus::Error> {
        if level > 100 {
            return Err(FdoErr::Failed("Brightness level must be 0-100".into()).into());
        }

        self.set_brightness_with_sync(&BacklightType::Primary, level)
            .await?;
        self.primary_brightness_changed(&ctxt).await?;

        Ok(())
    }

    #[zbus(property)]
    async fn screenpad_brightness(&self) -> Result<i32, FdoErr> {
        self.get_brightness_percent(&BacklightType::Screenpad)
    }

    #[zbus(property)]
    async fn set_screenpad_brightness(
        &self,
        #[zbus(signal_context)] ctxt: SignalEmitter<'_>,
        level: i32,
    ) -> Result<(), zbus::Error> {
        if level > 100 {
            return Err(FdoErr::Failed("Brightness level must be 0-100".into()).into());
        }

        self.set_brightness_with_sync(&BacklightType::Screenpad, level)
            .await?;
        self.screenpad_brightness_changed(&ctxt).await?;

        Ok(())
    }

    #[zbus(property)]
    async fn screenpad_power(&self) -> Result<bool, FdoErr> {
        if let Some(backlight) = self.get_backlight(&BacklightType::Screenpad) {
            let power = backlight.get_bl_power().map_err(|e| {
                warn!("Failed to get backlight power: {}", e);
                FdoErr::Failed(format!("Failed to get backlight power: {}", e))
            })?;
            Ok(power == 0)
        } else {
            Err(FdoErr::NotSupported("Screenpad backlight not found".into()))
        }
    }

    #[zbus(property)]
    async fn set_screenpad_power(
        &self,
        #[zbus(signal_context)] ctxt: SignalEmitter<'_>,
        power: bool,
    ) -> Result<(), zbus::Error> {
        if let Some(backlight) = self.get_backlight(&BacklightType::Screenpad) {
            backlight
                .set_bl_power(if power { 0 } else { 1 })
                .map_err(|e| {
                    warn!("Failed to set backlight power: {}", e);
                    FdoErr::Failed(format!("Failed to set backlight power: {}", e))
                })?;
            self.screenpad_power_changed(&ctxt).await?;
            Ok(())
        } else {
            Err(FdoErr::NotSupported("Screenpad backlight not found".into()).into())
        }
    }
}

impl crate::ZbusRun for CtrlBacklight {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, ASUS_ZBUS_PATH, server).await;
    }
}

impl crate::Reloadable for CtrlBacklight {
    async fn reload(&mut self) -> Result<(), RogError> {
        info!("Reloading backlight settings");
        Ok(())
    }
}
