use std::fs::{read_dir, File};
use std::io::Read;
use std::path::Path;

use crate::error::PlatformError;

const BASE_DIR: &str = "/sys/class/firmware-attributes/asus-armoury/attributes/";

fn read_i32(path: &Path) -> Result<i32, PlatformError> {
    if let Ok(mut f) = File::open(path) {
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        buf.trim()
            .parse::<i32>()
            .map_err(|_| PlatformError::ParseNum)
    } else {
        Err(PlatformError::ParseNum)
    }
}

fn read_string(path: &Path) -> Result<String, PlatformError> {
    let mut f = File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf.trim().to_string())
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub enum AttrValue {
    Integer(i32),
    String(String),
    EnumInt(Vec<i32>),
    EnumStr(Vec<String>),
    #[default]
    None,
}

#[derive(Debug, Default)]
pub struct Attribute {
    name: String,
    help: String,
    current_value: AttrValue,
    default_value: AttrValue,
    possible_values: AttrValue,
    min_value: AttrValue,
    max_value: AttrValue,
    scalar_increment: Option<i32>,
}

impl Attribute {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn help(&self) -> &str {
        &self.help
    }

    pub fn current_value(&self) -> &AttrValue {
        &self.current_value
    }

    pub fn default_value(&self) -> &AttrValue {
        &self.default_value
    }

    pub fn possible_values(&self) -> &AttrValue {
        &self.possible_values
    }

    pub fn min_value(&self) -> &AttrValue {
        &self.min_value
    }

    pub fn max_value(&self) -> &AttrValue {
        &self.max_value
    }

    pub fn scalar_increment(&self) -> Option<i32> {
        self.scalar_increment
    }

    fn read_values(
        base_path: &Path,
    ) -> (
        AttrValue,
        AttrValue,
        AttrValue,
        AttrValue,
        AttrValue,
        Option<i32>,
    ) {
        let current_value = match read_string(&base_path.join("current_value")) {
            Ok(val) => {
                if let Ok(int) = val.parse::<i32>() {
                    AttrValue::Integer(int)
                } else {
                    AttrValue::String(val)
                }
            }
            Err(_) => AttrValue::None,
        };

        let default_value = match read_string(&base_path.join("default_value")) {
            Ok(val) => {
                if let Ok(int) = val.parse::<i32>() {
                    AttrValue::Integer(int)
                } else {
                    AttrValue::String(val)
                }
            }
            Err(_) => AttrValue::None,
        };

        let possible_values = match read_string(&base_path.join("possible_values")) {
            Ok(val) => {
                if let Ok(int) = val.parse::<i32>() {
                    AttrValue::Integer(int)
                } else if val.contains(';') {
                    AttrValue::EnumInt(val.split(';').filter_map(|s| s.parse().ok()).collect())
                } else {
                    AttrValue::EnumStr(val.split(';').map(|s| s.to_string()).collect())
                }
            }
            Err(_) => AttrValue::None,
        };

        let min_value = read_i32(&base_path.join("min_value"))
            .ok()
            .map(AttrValue::Integer)
            .unwrap_or_default();
        let max_value = read_i32(&base_path.join("max_value"))
            .ok()
            .map(AttrValue::Integer)
            .unwrap_or_default();
        let scalar_increment = read_i32(&base_path.join("scalar_increment")).ok();

        (
            current_value,
            default_value,
            possible_values,
            min_value,
            max_value,
            scalar_increment,
        )
    }
}

pub struct FirmwareAttributes {
    attrs: Vec<Attribute>,
}

#[allow(clippy::new_without_default)]
impl FirmwareAttributes {
    pub fn new() -> Self {
        let mut attrs = Vec::new();
        if let Ok(dir) = read_dir(BASE_DIR) {
            for entry in dir.flatten() {
                let base_path = entry.path();
                let name = base_path.file_name().unwrap().to_string_lossy().to_string();
                let help = read_string(&base_path.join("display_name")).unwrap_or_default();

                let (
                    current_value,
                    default_value,
                    possible_values,
                    min_value,
                    max_value,
                    scalar_increment,
                ) = Attribute::read_values(&base_path);

                attrs.push(Attribute {
                    name,
                    help,
                    current_value,
                    default_value,
                    possible_values,
                    min_value,
                    max_value,
                    scalar_increment,
                });
            }
        }
        Self { attrs }
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attrs
    }

    pub fn attributes_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }
}

macro_rules! define_attribute_getters {
    ($($attr:ident),*) => {
        impl FirmwareAttributes {
            $(
                pub fn $attr(&self) -> Option<&Attribute> {
                    self.attrs.iter().find(|a| a.name() == stringify!($attr))
                }

                concat_idents::concat_idents!(attr_mut = $attr, _mut {
                    pub fn attr_mut(&mut self) -> Option<&mut Attribute> {
                        self.attrs.iter_mut().find(|a| a.name() == stringify!($attr))
                    }
                });
            )*
        }
    }
}

define_attribute_getters!(
    apu_mem,
    cores_performance,
    cores_efficiency,
    ppt_pl1_spl,
    ppt_pl2_sppt,
    ppt_apu_sppt,
    ppt_platform_sppt,
    ppt_fppt,
    nv_dynamic_boost,
    nv_temp_target,
    dgpu_base_tgp,
    dgpu_tgp,
    charge_mode,
    boot_sound,
    mcu_powersave,
    panel_od,
    panel_hd_mode,
    egpu_connected,
    egpu_enable,
    dgpu_disable,
    gpu_mux_mode,
    mini_led_mode
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_attributes() {
        let attrs = FirmwareAttributes::new();
        for attr in attrs.attributes() {
            dbg!(attr.name());
            match attr.name() {
                "nv_dynamic_boost" => {
                    assert!(!attr.help().is_empty());
                    assert!(matches!(attr.current_value, AttrValue::Integer(_)));
                    if let AttrValue::Integer(val) = attr.current_value {
                        assert_eq!(val, 5);
                    }
                    if let AttrValue::Integer(val) = attr.default_value {
                        assert_eq!(val, 25);
                    }
                    assert_eq!(attr.min_value, AttrValue::Integer(0));
                    assert_eq!(attr.max_value, AttrValue::Integer(25));
                }
                "boot_sound" => {
                    assert!(!attr.help().is_empty());
                    assert!(matches!(attr.current_value, AttrValue::Integer(0)));
                    // dbg!(attr.current_value());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_boot_sound() {
        let attrs = FirmwareAttributes::new();
        let attr = attrs
            .attributes()
            .iter()
            .find(|a| a.name() == "boot_sound")
            .unwrap();

        assert_eq!(attr.name(), "boot_sound");
        assert!(!attr.help().is_empty());
        assert!(matches!(attr.current_value(), AttrValue::Integer(_)));
        if let AttrValue::Integer(val) = attr.current_value() {
            assert_eq!(*val, 0); // assuming value is 0
        }
        // Check other members if applicable
    }
}
