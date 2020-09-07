use crate::config::Config;
use crate::config::Profile;
use asus_nb::profile::ProfileEvent;
use log::{error, info, warn};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

static FAN_TYPE_1_PATH: &str = "/sys/devices/platform/asus-nb-wmi/throttle_thermal_policy";
static FAN_TYPE_2_PATH: &str = "/sys/devices/platform/asus-nb-wmi/fan_boost_mode";
static AMD_BOOST_PATH: &str = "/sys/devices/system/cpu/cpufreq/boost";

pub struct CtrlFanAndCPU {
    path: &'static str,
}

use ::dbus::{nonblock::SyncConnection, tree::Signal};
use async_trait::async_trait;

#[async_trait]
impl crate::Controller for CtrlFanAndCPU {
    type A = ProfileEvent;

    /// Spawns two tasks which continuously check for changes
    fn spawn_task_loop(
        self,
        config: Arc<Mutex<Config>>,
        mut recv: Receiver<Self::A>,
        _: Option<Arc<SyncConnection>>,
        _: Option<Arc<Signal<()>>>,
    ) -> Vec<JoinHandle<()>> {
        let gate1 = Arc::new(Mutex::new(self));
        let gate2 = gate1.clone();
        let config1 = config.clone();
        // spawn an endless loop
        vec![
            tokio::spawn(async move {
                while let Some(event) = recv.recv().await {
                    let mut config = config1.lock().await;
                    let mut lock = gate1.lock().await;

                    config.read();
                    lock.handle_profile_event(&event, &mut config)
                        .unwrap_or_else(|err| warn!("{:?}", err));
                }
            }),
            // need to watch file path
            tokio::spawn(async move {
                loop {
                    tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
                    let mut lock = gate2.lock().await;
                    let mut config = config.lock().await;
                    lock.fan_mode_check_change(&mut config)
                        .unwrap_or_else(|err| warn!("fan_ctrl: {:?}", err));
                }
            }),
        ]
    }

    async fn reload_from_config(&mut self, config: &mut Config) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new().write(true).open(self.path)?;
        file.write_all(format!("{:?}\n", config.power_profile).as_bytes())
            .unwrap_or_else(|err| error!("Could not write to {}, {:?}", self.path, err));
        let profile = config.active_profile.clone();
        self.set_profile(&profile, config)?;
        info!(
            "Reloaded fan mode: {:?}",
            FanLevel::from(config.power_profile)
        );
        Ok(())
    }
}

impl CtrlFanAndCPU {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path = CtrlFanAndCPU::get_fan_path()?;
        info!("Device has thermal throttle control");
        Ok(CtrlFanAndCPU { path })
    }

    fn get_fan_path() -> Result<&'static str, std::io::Error> {
        if Path::new(FAN_TYPE_1_PATH).exists() {
            Ok(FAN_TYPE_1_PATH)
        } else if Path::new(FAN_TYPE_2_PATH).exists() {
            Ok(FAN_TYPE_2_PATH)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Fan mode not available",
            ))
        }
    }

    pub(super) fn fan_mode_check_change(
        &mut self,
        config: &mut Config,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new().read(true).open(self.path)?;
        let mut buf = [0u8; 1];
        file.read_exact(&mut buf)?;
        if let Some(num) = char::from(buf[0]).to_digit(10) {
            if config.power_profile != num as u8 {
                config.read();

                let mut i = config
                    .toggle_profiles
                    .iter()
                    .position(|x| x == &config.active_profile)
                    .map(|i| i + 1)
                    .unwrap_or(0);
                if i >= config.toggle_profiles.len() {
                    i = 0;
                }

                let new_profile = config
                    .toggle_profiles
                    .get(i)
                    .unwrap_or(&config.active_profile)
                    .clone();

                self.set_profile(&new_profile, config)?;

                info!("Profile was changed: {:?}", &new_profile);
            }
            return Ok(());
        }
        let err = std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Fan-level could not be parsed",
        );
        Err(Box::new(err))
    }

    pub(super) fn set_fan_mode(
        &mut self,
        preset: u8,
        config: &mut Config,
    ) -> Result<(), Box<dyn Error>> {
        let mode = config.active_profile.clone();
        let mut fan_ctrl = OpenOptions::new().write(true).open(self.path)?;
        config.read();
        let mut mode_config = config
            .power_profiles
            .get_mut(&mode)
            .ok_or_else(|| RogError::MissingProfile(mode.clone()))?;
        config.power_profile = preset;
        mode_config.fan_preset = preset;
        config.write();
        fan_ctrl
            .write_all(format!("{:?}\n", preset).as_bytes())
            .unwrap_or_else(|err| error!("Could not write to {}, {:?}", self.path, err));
        info!("Fan mode set to: {:?}", FanLevel::from(preset));
        self.set_pstate_for_fan_mode(&mode, config)?;
        self.set_fan_curve_for_fan_mode(&mode, config)?;
        Ok(())
    }

    fn handle_profile_event(
        &mut self,
        event: &ProfileEvent,
        config: &mut Config,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            ProfileEvent::ChangeMode(mode) => {
                self.set_fan_mode(*mode, config)?;
            }
            ProfileEvent::Cli(command) => {
                let profile_key = match command.profile.as_ref() {
                    Some(k) => k.clone(),
                    None => config.active_profile.clone(),
                };

                let mut profile = if command.create {
                    config
                        .power_profiles
                        .entry(profile_key.clone())
                        .or_insert_with(|| Profile::default())
                } else {
                    config
                        .power_profiles
                        .get_mut(&profile_key)
                        .ok_or_else(|| RogError::MissingProfile(profile_key.clone()))?
                };

                if command.turbo {
                    profile.no_turbo = false;
                }
                if command.no_turbo {
                    profile.no_turbo = true;
                }
                if let Some(min_perc) = command.min_percentage {
                    profile.min_percentage = min_perc;
                }
                if let Some(max_perc) = command.max_percentage {
                    profile.max_percentage = max_perc;
                }
                if let Some(ref preset) = command.preset {
                    profile.fan_preset = preset.into();
                }
                if let Some(ref curve) = command.curve {
                    profile.fan_curve = Some(curve.clone());
                }

                self.set_profile(&profile_key, config)?;
            }
        }
        Ok(())
    }

    fn set_profile(&mut self, profile: &str, config: &mut Config) -> Result<(), Box<dyn Error>> {
        let mode_config = config
            .power_profiles
            .get(profile)
            .ok_or_else(|| RogError::MissingProfile(profile.into()))?;
        let mut fan_ctrl = OpenOptions::new().write(true).open(self.path)?;
        fan_ctrl
            .write_all(format!("{:?}\n", mode_config.fan_preset).as_bytes())
            .unwrap_or_else(|err| error!("Could not write to {}, {:?}", self.path, err));
        config.power_profile = mode_config.fan_preset;

        self.set_pstate_for_fan_mode(profile, config)?;
        self.set_fan_curve_for_fan_mode(profile, config)?;

        config.active_profile = profile.into();

        config.write();
        Ok(())
    }

    fn set_pstate_for_fan_mode(
        &self,
        // mode: FanLevel,
        mode: &str,
        config: &mut Config,
    ) -> Result<(), Box<dyn Error>> {
        info!("Setting pstate");
        let mode_config = config
            .power_profiles
            .get(mode)
            .ok_or_else(|| RogError::MissingProfile(mode.into()))?;

        // Set CPU pstate
        if let Ok(pstate) = intel_pstate::PState::new() {
            pstate.set_min_perf_pct(mode_config.min_percentage)?;
            pstate.set_max_perf_pct(mode_config.max_percentage)?;
            pstate.set_no_turbo(mode_config.no_turbo)?;
            info!(
                "Intel CPU Power: min: {:?}%, max: {:?}%, turbo: {:?}",
                mode_config.min_percentage, mode_config.max_percentage, !mode_config.no_turbo
            );
        } else {
            info!("Setting pstate for AMD CPU");
            // must be AMD CPU
            let mut file = OpenOptions::new()
                .write(true)
                .open(AMD_BOOST_PATH)
                .map_err(|err| {
                    warn!("Failed to open AMD boost: {:?}", err);
                    err
                })?;

            let boost = if mode_config.no_turbo { "0" } else { "1" }; // opposite of Intel
            file.write_all(boost.as_bytes())
                .unwrap_or_else(|err| error!("Could not write to {}, {:?}", AMD_BOOST_PATH, err));
            info!("AMD CPU Turbo: {:?}", boost);
        }
        Ok(())
    }

    fn set_fan_curve_for_fan_mode(
        &self,
        // mode: FanLevel,
        mode: &str,
        config: &Config,
    ) -> Result<(), Box<dyn Error>> {
        let mode_config = &config
            .power_profiles
            .get(mode)
            .ok_or_else(|| RogError::MissingProfile(mode.into()))?;

        if let Some(ref curve) = mode_config.fan_curve {
            use rog_fan_curve::{Board, Fan};
            if let Some(board) = Board::from_board_name() {
                curve.apply(board, Fan::Cpu)?;
                curve.apply(board, Fan::Gpu)?;
            } else {
                warn!("Fan curve unsupported on this board.")
            }
        }

        Ok(())
    }
}

use crate::error::RogError;

#[derive(Debug)]
pub enum FanLevel {
    Normal,
    Boost,
    Silent,
}

impl FromStr for FanLevel {
    type Err = RogError;

    fn from_str(s: &str) -> Result<Self, RogError> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(FanLevel::Normal),
            "boost" => Ok(FanLevel::Boost),
            "silent" => Ok(FanLevel::Silent),
            _ => Err(RogError::ParseFanLevel),
        }
    }
}

impl From<u8> for FanLevel {
    fn from(n: u8) -> Self {
        match n {
            0 => FanLevel::Normal,
            1 => FanLevel::Boost,
            2 => FanLevel::Silent,
            _ => FanLevel::Normal,
        }
    }
}

impl From<FanLevel> for u8 {
    fn from(n: FanLevel) -> Self {
        match n {
            FanLevel::Normal => 0,
            FanLevel::Boost => 1,
            FanLevel::Silent => 2,
        }
    }
}
