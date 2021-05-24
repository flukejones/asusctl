use rog_fan_curve::{Curve, Fan};
use serde_derive::{Deserialize, Serialize};
use std::io::Write;
use std::{fs::OpenOptions, path::Path, str::FromStr};
#[cfg(feature = "dbus")]
use zvariant_derive::Type;

use crate::{error::ProfileError, AMD_BOOST_PATH, FAN_TYPE_1_PATH, FAN_TYPE_2_PATH};

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
    pub min_percentage: u8,
    pub max_percentage: u8,
    pub turbo: bool,
    pub fan_preset: FanLevel,
    pub fan_curve: String,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            name: "new".into(),
            min_percentage: 0,
            max_percentage: 100,
            turbo: false,
            fan_preset: FanLevel::Normal,
            fan_curve: "".to_string(),
        }
    }
}

impl Profile {
    pub fn new(
        name: String,
        min_percentage: u8,
        max_percentage: u8,
        turbo: bool,
        fan_preset: FanLevel,
        fan_curve: String,
    ) -> Self {
        Profile {
            name,
            min_percentage,
            max_percentage,
            turbo,
            fan_preset,
            fan_curve,
        }
    }

    pub fn get_intel_supported() -> bool {
        intel_pstate::PState::new().is_ok()
    }

    pub fn get_fan_path() -> Result<&'static str, ProfileError> {
        if Path::new(FAN_TYPE_1_PATH).exists() {
            Ok(FAN_TYPE_1_PATH)
        } else if Path::new(FAN_TYPE_2_PATH).exists() {
            Ok(FAN_TYPE_2_PATH)
        } else {
            Err(ProfileError::NotSupported)
        }
    }

    pub fn set_system_pstate(&self) -> Result<(), ProfileError> {
        // Set CPU pstate
        if let Ok(pstate) = intel_pstate::PState::new() {
            pstate.set_min_perf_pct(self.min_percentage)?;
            pstate.set_max_perf_pct(self.max_percentage)?;
            pstate.set_no_turbo(!self.turbo)?;
        } else {
            // must be AMD CPU
            let mut file = OpenOptions::new()
                .write(true)
                .open(AMD_BOOST_PATH)
                .map_err(|err| ProfileError::Path(AMD_BOOST_PATH.into(), err))?;

            let boost = if self.turbo { "1" } else { "0" }; // opposite of Intel
            file.write_all(boost.as_bytes())
                .map_err(|err| ProfileError::Write(AMD_BOOST_PATH.into(), err))?;
        }
        Ok(())
    }

    pub fn set_system_fan_mode(&self) -> Result<(), ProfileError> {
        let path = Profile::get_fan_path()?;
        let mut fan_ctrl = OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|err| ProfileError::Path(path.into(), err))?;
        fan_ctrl
            .write_all(format!("{}\n", <u8>::from(self.fan_preset)).as_bytes())
            .map_err(|err| ProfileError::Write(path.into(), err))?;
        Ok(())
    }

    pub fn set_system_fan_curve(&self) -> Result<(), ProfileError> {
        if !self.fan_curve.is_empty() {
            if let Ok(curve) = Profile::parse_fan_curve(&self.fan_curve) {
                use rog_fan_curve::Board;
                if let Some(board) = Board::from_board_name() {
                    curve.apply(board, Fan::Cpu)?;
                    curve.apply(board, Fan::Gpu)?;
                }
            }
        }

        Ok(())
    }

    pub fn set_system_all(&self) -> Result<(), ProfileError> {
        self.set_system_pstate()?;
        if !self.fan_curve.is_empty() {
            self.set_system_fan_mode()?;
        } else {
            self.set_system_fan_curve()?;
        }
        Ok(())
    }

    fn parse_fan_curve(data: &str) -> Result<Curve, String> {
        let curve = Curve::from_config_str(data)?;
        if let Err(err) = curve.check_safety(Fan::Cpu) {
            return Err(format!("Unsafe curve {:?}", err));
        }
        if let Err(err) = curve.check_safety(Fan::Gpu) {
            return Err(format!("Unsafe curve {:?}", err));
        }
        Ok(curve)
    }
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum FanLevel {
    Normal,
    Boost,
    Silent,
}

impl FromStr for FanLevel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(FanLevel::Normal),
            "boost" => Ok(FanLevel::Boost),
            "silent" => Ok(FanLevel::Silent),
            _ => Err("Invalid fan level"),
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

impl From<&FanLevel> for u8 {
    fn from(n: &FanLevel) -> Self {
        match n {
            FanLevel::Normal => 0,
            FanLevel::Boost => 1,
            FanLevel::Silent => 2,
        }
    }
}
