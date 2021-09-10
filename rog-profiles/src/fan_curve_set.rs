use serde_derive::{Deserialize, Serialize};

use udev::Device;
#[cfg(feature = "dbus")]
use zvariant_derive::Type;

use crate::{error::ProfileError, write_to_fan, FanCurvePU};

pub fn pwm_str(fan: char, index: char) -> String {
    let mut buf = "pwm1_auto_point1_pwm".to_string();
    unsafe {
        let tmp = buf.as_bytes_mut();
        tmp[3] = fan as u8;
        tmp[15] = index as u8;
    }
    buf
}

pub fn temp_str(fan: char, index: char) -> String {
    let mut buf = "pwm1_auto_point1_temp".to_string();
    unsafe {
        let tmp = buf.as_bytes_mut();
        tmp[3] = fan as u8;
        tmp[15] = index as u8;
    }
    buf
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct CurveData {
    pub fan: FanCurvePU,
    pub pwm: [u8; 8],
    pub temp: [u8; 8],
}

/// A `FanCurveSet` contains both CPU and GPU fan curve data
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FanCurveSet {
    pub cpu: CurveData,
    pub gpu: CurveData,
}

impl Default for FanCurveSet {
    fn default() -> Self {
        Self {
            cpu: CurveData {
                fan: FanCurvePU::CPU,
                pwm: [0u8; 8],
                temp: [0u8; 8],
            },
            gpu: CurveData {
                fan: FanCurvePU::GPU,
                pwm: [0u8; 8],
                temp: [0u8; 8],
            },
        }
    }
}

impl FanCurveSet {
    pub fn get_device() -> Result<Device, ProfileError> {
        let mut enumerator = udev::Enumerator::new()?;
        enumerator.match_subsystem("hwmon")?;

        for device in enumerator.scan_devices().unwrap() {
            if device.parent_with_subsystem("platform").unwrap().is_some() {
                if let Some(name) = device.attribute_value("name") {
                    if name == "asus_custom_fan_curve" {
                        return Ok(device);
                    }
                }
            }
        }
        Err(ProfileError::NotSupported)
    }

    pub fn is_supported() -> Result<bool, ProfileError> {
        if Self::get_device().is_ok() {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn new() -> Result<(Self, Device), ProfileError> {
        if let Ok(device) = Self::get_device() {
            let mut fans = Self {
                cpu: CurveData::default(),
                gpu: CurveData::default(),
            };

            fans.cpu.fan = FanCurvePU::CPU;
            fans.cpu.fan = FanCurvePU::GPU;

            fans.read_from_device(&device);

            return Ok((fans, device));
        }

        Err(ProfileError::NotSupported)
    }

    fn set_val_from_attr(tmp: &str, device: &Device, buf: &mut [u8; 8]) {
        if let Some(n) = tmp.chars().nth(15) {
            let i = n.to_digit(10).unwrap() as usize;
            let d = device.attribute_value(tmp).unwrap();
            let d: u8 = d.to_string_lossy().parse().unwrap();
            buf[i - 1] = d;
        }
    }

    pub fn read_from_device(&mut self, device: &Device) {
        for attr in device.attributes() {
            let tmp = attr.name().to_string_lossy();
            if tmp.starts_with("pwm1") && tmp.ends_with("_temp") {
                Self::set_val_from_attr(tmp.as_ref(), device, &mut self.cpu.temp)
            }
            if tmp.starts_with("pwm1") && tmp.ends_with("_pwm") {
                Self::set_val_from_attr(tmp.as_ref(), device, &mut self.cpu.pwm)
            }
            if tmp.starts_with("pwm2") && tmp.ends_with("_temp") {
                Self::set_val_from_attr(tmp.as_ref(), device, &mut self.gpu.temp)
            }
            if tmp.starts_with("pwm2") && tmp.ends_with("_pwm") {
                Self::set_val_from_attr(tmp.as_ref(), device, &mut self.gpu.pwm)
            }
        }
    }

    pub fn write_cpu_fan(&self, device: &mut Device) {
        write_to_fan(&self.cpu, '1', device);
    }

    pub fn write_gpu_fan(&self, device: &mut Device) {
        write_to_fan(&self.gpu, '2', device);
    }
}
