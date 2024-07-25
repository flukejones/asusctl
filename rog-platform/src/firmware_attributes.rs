// Firmware attribute interfaces
// - current_value
// - default_value
// - display_name
// - default_value
// - possible_values
// - max_value
// - min_value
// - scalar_increment
// - type

use std::{
    fs::{read_dir, File},
    io::Read,
    path::{Path, PathBuf},
};

use log::error;

use crate::error::PlatformError;

const BASE_DIR: &str = "/sys/class/firmware-attributes/asus-bioscfg/attributes/";

fn read_i32(path: &Path) -> Result<i32, PlatformError> {
    if let Ok(mut f) = File::open(path) {
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        buf = buf.trim_end().to_string();
        if let Ok(int) = buf.parse::<i32>() {
            return Ok(int);
        }
    }
    Err(PlatformError::ParseNum)
}

fn read_string(path: &Path) -> Result<String, PlatformError> {
    if let Ok(mut f) = File::open(path) {
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        buf = buf.trim_end().to_string();
        return Ok(buf);
    }
    Err(PlatformError::ParseNum)
}

fn attr_path_if_exists(mut base_path: PathBuf, attr: &str) -> Option<PathBuf> {
    base_path.push(attr);
    if base_path.exists() {
        return Some(base_path.clone());
    }
    None
}

#[derive(Debug, Default)]
pub struct AttrInteger {
    current: PathBuf,
    default: Option<PathBuf>,
    min: Option<PathBuf>,
    max: Option<PathBuf>,
    scalar_inc: Option<PathBuf>,
}

impl AttrInteger {
    pub fn current_value(&self) -> Result<i32, PlatformError> {
        read_i32(&self.current)
    }

    fn read_i32(path: Option<&PathBuf>) -> Result<Option<i32>, PlatformError> {
        if let Some(path) = path {
            let int = read_i32(path)?;
            return Ok(Some(int));
        }
        Ok(None)
    }

    pub fn default_value(&self) -> Result<Option<i32>, PlatformError> {
        Self::read_i32(self.default.as_ref())
    }

    pub fn min_value(&self) -> Result<Option<i32>, PlatformError> {
        Self::read_i32(self.min.as_ref())
    }

    pub fn max_value(&self) -> Result<Option<i32>, PlatformError> {
        Self::read_i32(self.max.as_ref())
    }

    pub fn scalar_increment(&self) -> Result<Option<i32>, PlatformError> {
        Self::read_i32(self.scalar_inc.as_ref())
    }
}

#[derive(Debug, Default)]
pub struct AttEnumInteger {
    current: PathBuf,
    default: Option<PathBuf>,
    possible: Option<PathBuf>,
}

impl AttEnumInteger {
    pub fn current_value(&self) -> Result<i32, PlatformError> {
        read_i32(&self.current)
    }

    pub fn default_value(&self) -> Result<Option<i32>, PlatformError> {
        if let Some(path) = self.default.as_ref() {
            let int = read_i32(path)?;
            return Ok(Some(int));
        }
        Ok(None)
    }

    pub fn possible_values(&self) -> Vec<i32> {
        let mut output = Vec::new();
        if let Some(path) = self.possible.as_ref() {
            if let Ok(string) = read_string(path) {
                for n in string.split(';') {
                    match n.parse::<i32>() {
                        Ok(n) => output.push(n),
                        Err(e) => error!("Couldn't parse num: {e:?}"),
                    }
                }
            }
        }

        output
    }
}

#[derive(Debug, Default)]
pub struct AttEnumString {
    current: PathBuf,
    default: Option<PathBuf>,
    possible: Option<PathBuf>,
}

impl AttEnumString {
    pub fn current_value(&self) -> Result<String, PlatformError> {
        read_string(&self.current)
    }

    pub fn default_value(&self) -> Result<Option<String>, PlatformError> {
        if let Some(path) = self.default.as_ref() {
            let string = read_string(path)?;
            return Ok(Some(string));
        }
        Ok(None)
    }

    pub fn possible_values(&self) -> Vec<String> {
        let mut output = Vec::new();
        if let Some(path) = self.possible.as_ref() {
            if let Ok(string) = read_string(path) {
                for n in string.split(';') {
                    if !n.is_empty() {
                        output.push(n.to_owned());
                    }
                }
            }
        }

        output
    }
}

#[derive(Debug, Default)]
pub enum AttrType {
    Integer(AttrInteger),
    EnumInt(AttEnumInteger),
    EnumStr(AttEnumString),
    #[default]
    Unknown,
}

#[derive(Debug, Default)]
pub struct Attribute {
    name: String,
    help: String,
    _base_path: PathBuf,
    attr_type: AttrType,
}

impl Attribute {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn help(&self) -> &str {
        self.help.as_str()
    }
}

pub fn get_attributes() -> Vec<Attribute> {
    let mut attrs = Vec::new();

    let dir = read_dir(BASE_DIR).unwrap();
    dir.for_each(|d| {
        if let Ok(base_dir) = d {
            let mut attr_path = base_dir.path();
            let mut attr = Attribute {
                _base_path: base_dir.path(),
                attr_type: AttrType::Unknown,
                ..Default::default()
            };

            // TYPE
            attr_path.push("type");
            let mut buf = String::new();
            if let Ok(mut f) = File::open(&attr_path) {
                f.read_to_string(&mut buf).unwrap();
                buf = buf.trim_end().to_string();

                attr_path.pop();
                let mut current = attr_path.clone();
                current.push("current_value");

                match buf.to_lowercase().as_str() {
                    "integer" => {
                        attr.attr_type = AttrType::Integer(AttrInteger {
                            current,
                            default: attr_path_if_exists(attr_path.clone(), "default_value"),
                            min: attr_path_if_exists(attr_path.clone(), "min_value"),
                            max: attr_path_if_exists(attr_path.clone(), "max_value"),
                            scalar_inc: attr_path_if_exists(attr_path.clone(), "scalar_increment"),
                        })
                    }
                    _ => {
                        // Check what the current_value type is
                        if let Ok(mut f) = File::open(&current) {
                            let mut buf = String::new();
                            f.read_to_string(&mut buf).unwrap();
                            buf = buf.trim_end().to_string();
                            if buf.parse::<i32>().is_ok() {
                                attr.attr_type = AttrType::EnumInt(AttEnumInteger {
                                    current,
                                    default: attr_path_if_exists(
                                        attr_path.clone(),
                                        "default_value",
                                    ),
                                    possible: attr_path_if_exists(
                                        attr_path.clone(),
                                        "possible_values",
                                    ),
                                });
                            } else {
                                attr.attr_type = AttrType::EnumStr(AttEnumString {
                                    current,
                                    default: attr_path_if_exists(
                                        attr_path.clone(),
                                        "default_value",
                                    ),
                                    possible: attr_path_if_exists(
                                        attr_path.clone(),
                                        "possible_values",
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            // DISPLAY_NAME
            attr_path.push("display_name");
            if let Ok(res) = read_string(&attr_path) {
                attr.help = res;
            }
            // DISPLAY_NAME
            attr_path.pop();
            attr.name = attr_path.file_name().unwrap().to_string_lossy().to_string();

            attrs.push(attr);
        }
    });

    attrs
}

#[cfg(test)]
mod tests {
    use super::get_attributes;

    #[test]
    fn find_attributes() {
        let attrs = get_attributes();
        for a in attrs {
            dbg!(&a);
        }
    }
}
