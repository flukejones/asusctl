use serde_derive::{Deserialize, Serialize};

use udev::Device;
#[cfg(feature = "dbus")]
use zvariant_derive::Type;

use crate::error::ProfileError;

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
    pub pwm: [u8; 8],
    pub temp: [u8; 8],
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct FanCurveSet {
    pub cpu: CurveData,
    pub gpu: CurveData,
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

    pub fn new() -> Result<(Self, Device), ProfileError> {
        if let Ok(device) = Self::get_device() {
            let mut fans = Self {
                cpu: CurveData::default(),
                gpu: CurveData::default(),
            };
            fans.init_from_device(&device);
            return Ok((fans, device));
        }

        Err(ProfileError::NotSupported)
    }

    pub fn is_supported() -> Result<bool, ProfileError> {
        if Self::get_device().is_ok() {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn update_from_device(&mut self, device: &Device) {
        self.init_from_device(device);
    }

    fn set_val_from_attr(tmp: &str, device: &Device, buf: &mut [u8; 8]) {
        if let Some(n) = tmp.chars().nth(15) {
            let i = n.to_digit(10).unwrap() as usize;
            let d = device.attribute_value(tmp).unwrap();
            let d: u8 = d.to_string_lossy().parse().unwrap();
            buf[i - 1] = d;
        }
    }

    pub fn init_from_device(&mut self, device: &Device) {
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

    fn write_to_fan(curve: &CurveData, pwm_num: char, device: &mut Device) {
        let mut pwm = "pwmN_auto_pointN_pwm".to_string();

        for (index,out) in curve.pwm.iter().enumerate() {
            unsafe {
                let buf = pwm.as_bytes_mut();
                buf[3] = pwm_num as u8;
                // Should be quite safe to unwrap as we're not going over 8
                buf[15] = char::from_digit(index as u32, 10).unwrap() as u8;
            }
            let out = out.to_string();
            device.set_attribute_value(&pwm, &out).unwrap();
        }

        let mut pwm = "pwmN_auto_pointN_temp".to_string();

        for (index,out) in curve.temp.iter().enumerate() {
            unsafe {
                let buf = pwm.as_bytes_mut();
                buf[3] = pwm_num as u8;
                // Should be quite safe to unwrap as we're not going over 8
                buf[15] = char::from_digit(index as u32, 10).unwrap() as u8;
            }
            let out = out.to_string();
            device.set_attribute_value(&pwm, &out).unwrap();
        }
    }

    pub fn write_cpu_fan(&self, device: &mut Device) {
        Self::write_to_fan(&self.cpu, '1', device);
    }

    pub fn write_gpu_fan(&self, device: &mut Device) {
        Self::write_to_fan(&self.gpu, '2', device);
    }
}
