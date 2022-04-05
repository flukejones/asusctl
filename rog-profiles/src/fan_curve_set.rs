use serde_derive::{Deserialize, Serialize};

use udev::Device;
#[cfg(feature = "dbus")]
use zvariant_derive::Type;

use crate::{error::ProfileError, FanCurvePU};

pub(crate) fn pwm_str(fan: char, index: usize) -> String {
    let mut buf = "pwm1_auto_point1_pwm".to_string();
    unsafe {
        let tmp = buf.as_bytes_mut();
        tmp[3] = fan as u8;
        tmp[15] = char::from_digit(index as u32 + 1, 10).unwrap() as u8;
    }
    buf
}

pub(crate) fn temp_str(fan: char, index: usize) -> String {
    let mut buf = "pwm1_auto_point1_temp".to_string();
    unsafe {
        let tmp = buf.as_bytes_mut();
        tmp[3] = fan as u8;
        tmp[15] = char::from_digit(index as u32 + 1, 10).unwrap() as u8;
    }
    buf
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct CurveData {
    pub(crate) fan: FanCurvePU,
    pub(crate) pwm: [u8; 8],
    pub(crate) temp: [u8; 8],
}

impl std::str::FromStr for CurveData {
    type Err = ProfileError;

    /// Parse a string to the correct values that the fan curve kernel driver expects
    ///
    /// If the fan curve is given with percentage char '%' then the fan power values are converted
    /// otherwise the expected fan power range is 0-255.
    ///
    /// Temperature range is 0-255 in degrees C. You don't want to be setting over 100.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut temp = [0u8; 8];
        let mut pwm = [0u8; 8];
        let mut temp_prev = 0;
        let mut pwm_prev = 0;
        let mut percentages = false;

        for (index, value) in input.split(',').enumerate() {
            for (select, num) in value.splitn(2, |c| c == 'c' || c == ':').enumerate() {
                if num.contains('%') {
                    percentages = true;
                }
                let r = num.trim_matches(|c| c == 'c' || c == ':' || c == '%');
                let r = r.parse::<u8>().map_err(ProfileError::ParseFanCurveDigit)?;

                if select == 0 {
                    if temp_prev > r {
                        return Err(ProfileError::ParseFanCurvePrevHigher(
                            "temperature",
                            temp_prev,
                            r,
                        ));
                    }
                    temp_prev = r;
                    temp[index] = r;
                } else {
                    let mut p = r;
                    if percentages {
                        p *= 255 / 100;
                        if r > 100 {
                            return Err(ProfileError::ParseFanCurvePercentOver100(r));
                        }
                    }
                    if pwm_prev > p {
                        return Err(ProfileError::ParseFanCurvePrevHigher(
                            "percentage",
                            pwm_prev,
                            p,
                        ));
                    }
                    pwm_prev = p;
                    pwm[index] = p;
                }
            }
        }
        Ok(Self {
            fan: FanCurvePU::CPU,
            pwm,
            temp,
        })
    }
}

impl CurveData {
    pub fn set_fan(&mut self, fan: FanCurvePU) {
        self.fan = fan;
    }

    fn set_val_from_attr(tmp: &str, device: &Device, buf: &mut [u8; 8]) {
        if let Some(n) = tmp.chars().nth(15) {
            let i = n.to_digit(10).unwrap() as usize;
            let d = device.attribute_value(tmp).unwrap();
            let d: u8 = d.to_string_lossy().parse().unwrap();
            buf[i - 1] = d;
        }
    }

    fn read_from_device(&mut self, device: &Device) {
        for attr in device.attributes() {
            let tmp = attr.name().to_string_lossy();
            if tmp.starts_with("pwm1") && tmp.ends_with("_temp") {
                Self::set_val_from_attr(tmp.as_ref(), device, &mut self.temp)
            }
            if tmp.starts_with("pwm1") && tmp.ends_with("_pwm") {
                Self::set_val_from_attr(tmp.as_ref(), device, &mut self.pwm)
            }
        }
    }

    fn init_if_zeroed(&mut self, device: &mut Device) -> std::io::Result<()> {
        if self.pwm == [0u8; 8] && self.temp == [0u8; 8] {
            // Need to reset the device to defaults to get the proper profile defaults
            match self.fan {
                FanCurvePU::CPU => device.set_attribute_value("pwm1_enable", "3")?,
                FanCurvePU::GPU => device.set_attribute_value("pwm2_enable", "3")?,
            };
            self.read_from_device(device);
        }
        Ok(())
    }

    /// Write this curve to the device fan specified by `self.fan`
    fn write_to_device(&self, device: &mut Device, enable: bool) -> std::io::Result<()> {
        let pwm_num = match self.fan {
            FanCurvePU::CPU => '1',
            FanCurvePU::GPU => '2',
        };
        let enable = if enable { "1" } else { "2" };

        for (index, out) in self.pwm.iter().enumerate() {
            let pwm = pwm_str(pwm_num, index);
            device.set_attribute_value(&pwm, &out.to_string())?;
        }

        for (index, out) in self.temp.iter().enumerate() {
            let temp = temp_str(pwm_num, index);
            device.set_attribute_value(&temp, &out.to_string())?;
        }

        // Enable must be done *after* all points are written
        match self.fan {
            FanCurvePU::CPU => device.set_attribute_value("pwm1_enable", enable)?,
            FanCurvePU::GPU => device.set_attribute_value("pwm2_enable", enable)?,
        };

        Ok(())
    }
}

/// A `FanCurveSet` contains both CPU and GPU fan curve data
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FanCurveSet {
    pub(crate) enabled: bool,
    pub(crate) cpu: CurveData,
    pub(crate) gpu: CurveData,
}

impl Default for FanCurveSet {
    fn default() -> Self {
        Self {
            enabled: false,
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
    pub(crate) fn read_cpu_from_device(&mut self, device: &Device) {
        self.cpu.read_from_device(device);
    }

    pub(crate) fn read_gpu_from_device(&mut self, device: &Device) {
        self.gpu.read_from_device(device);
    }

    pub(crate) fn write_cpu_fan(&mut self, device: &mut Device) -> std::io::Result<()> {
        self.cpu.init_if_zeroed(device)?;
        self.cpu.write_to_device(device, self.enabled)
    }

    pub(crate) fn write_gpu_fan(&mut self, device: &mut Device) -> std::io::Result<()> {
        self.gpu.init_if_zeroed(device)?;
        self.gpu.write_to_device(device, self.enabled)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn curve_data_from_str() {
        let curve =
            CurveData::from_str("30c:1%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%")
                .unwrap();
        assert_eq!(curve.fan, FanCurvePU::CPU);
        assert_eq!(curve.temp, [30, 49, 59, 69, 79, 89, 99, 109]);
        assert_eq!(curve.pwm, [1, 2, 3, 4, 31, 49, 56, 58]);
    }

    #[test]
    fn curve_data_from_str_simple() {
        let curve = CurveData::from_str("30:1,49:2,59:3,69:4,79:31,89:49,99:56,109:58").unwrap();
        assert_eq!(curve.fan, FanCurvePU::CPU);
        assert_eq!(curve.temp, [30, 49, 59, 69, 79, 89, 99, 109]);
        assert_eq!(curve.pwm, [1, 2, 3, 4, 31, 49, 56, 58]);
    }

    #[test]
    fn curve_data_from_str_invalid_pwm() {
        let curve =
            CurveData::from_str("30c:4%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%");
        assert!(&curve.is_err());
        assert!(matches!(
            curve,
            Err(ProfileError::ParseFanCurvePrevHigher(_, _, _))
        ));
    }

    #[test]
    fn check_pwm_str() {
        assert_eq!(pwm_str('1', 0), "pwm1_auto_point1_pwm");
        assert_eq!(pwm_str('1', 4), "pwm1_auto_point5_pwm");
        assert_eq!(pwm_str('1', 7), "pwm1_auto_point8_pwm");
    }

    #[test]
    fn check_temp_str() {
        assert_eq!(temp_str('1', 0), "pwm1_auto_point1_temp");
        assert_eq!(temp_str('1', 4), "pwm1_auto_point5_temp");
        assert_eq!(temp_str('1', 7), "pwm1_auto_point8_temp");
    }
}
