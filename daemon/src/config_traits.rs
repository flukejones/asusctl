use std::fs::{create_dir, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use log::{error, warn};
use ron::ser::PrettyConfig;
use serde::de::DeserializeOwned;
use serde::Serialize;

const CONFIG_PATH_BASE: &str = "/etc/asusd/";

/// Config file helper traits. Only `new()` and `file_name()` are required to be
/// implemented, the rest are intended to be free methods.
pub trait StdConfig
where
    Self: Serialize + DeserializeOwned,
{
    fn new() -> Self;

    fn file_name() -> &'static str;

    fn file_path() -> PathBuf {
        let mut config = PathBuf::from(CONFIG_PATH_BASE);
        if !config.exists() {
            create_dir(config.as_path())
                .unwrap_or_else(|e| panic!("Could not create {CONFIG_PATH_BASE} {e}"));
        }
        config.push(Self::file_name());
        config
    }

    fn file_open() -> File {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(Self::file_path())
            .unwrap_or_else(|e| panic!("Could not open {:?} {e}", Self::file_path()))
    }

    fn read(&mut self) {
        let mut file = match OpenOptions::new().read(true).open(Self::file_path()) {
            Ok(data) => data,
            Err(err) => {
                error!("Error reading {:?}: {}", Self::file_path(), err);
                return;
            }
        };
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                warn!("File is empty {:?}", Self::file_path());
            } else if let Ok(data) = ron::from_str(&buf) {
                *self = data;
            } else if let Ok(data) = serde_json::from_str(&buf) {
                *self = data;
            } else {
                warn!("Could not deserialise {:?}", Self::file_path());
            }
        }
    }

    fn write(&self) {
        let mut file = match File::create(Self::file_path()) {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Couldn't overwrite config {:?}, error: {e}",
                    Self::file_path()
                );
                return;
            }
        };
        let ron = match ron::ser::to_string_pretty(&self, PrettyConfig::new().depth_limit(4)) {
            Ok(data) => data,
            Err(e) => {
                error!("Parse {:?} to RON failed, error: {e}", Self::file_path());
                return;
            }
        };
        file.write_all(ron.as_bytes())
            .unwrap_or_else(|err| error!("Could not write config: {}", err));
    }

    /// Renames the existing file to `<file>-old`
    fn rename_file_old() {
        warn!(
            "Renaming {} to {}-old and recreating config",
            Self::file_name(),
            Self::file_name()
        );
        let cfg_old = Self::file_path().to_string_lossy().to_string() + "-old";
        std::fs::rename(Self::file_path(), cfg_old).unwrap_or_else(|err| {
            error!(
                "Could not rename. Please remove {} then restart service: Error {}",
                Self::file_name(),
                err
            )
        });
    }
}

pub trait StdConfigLoad1<T>
where
    T: StdConfig + DeserializeOwned + Serialize,
{
    fn load() -> T {
        let mut file = T::file_open();
        let mut buf = String::new();
        let config: T;
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                config = T::new();
            } else if let Ok(data) = ron::from_str(&buf) {
                config = data;
            } else if let Ok(data) = serde_json::from_str(&buf) {
                config = data;
            } else {
                T::rename_file_old();
                config = T::new();
            }
        } else {
            config = T::new();
        }
        config.write();
        config
    }
}

pub trait StdConfigLoad2<T1, T2>
where
    T1: StdConfig + DeserializeOwned + Serialize,
    T2: DeserializeOwned + Into<T1>,
{
    fn load() -> T1 {
        let mut file = T1::file_open();
        let mut buf = String::new();
        let config: T1;
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                config = T1::new();
            } else if let Ok(data) = ron::from_str(&buf) {
                config = data;
            } else if let Ok(data) = serde_json::from_str(&buf) {
                config = data;
            } else if let Ok(data) = serde_json::from_str::<T2>(&buf) {
                config = data.into();
            } else {
                T1::rename_file_old();
                config = T1::new();
            }
        } else {
            config = T1::new();
        }
        config.write();
        config
    }
}

pub trait StdConfigLoad3<T1, T2, T3>
where
    T1: StdConfig + DeserializeOwned + Serialize,
    T2: DeserializeOwned + Into<T1>,
    T3: DeserializeOwned + Into<T1>,
{
    fn load() -> T1 {
        let mut file = T1::file_open();
        let mut buf = String::new();
        let config: T1;
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                config = T1::new();
            } else if let Ok(data) = ron::from_str(&buf) {
                config = data;
            } else if let Ok(data) = serde_json::from_str(&buf) {
                config = data;
            } else if let Ok(data) = serde_json::from_str::<T2>(&buf) {
                config = data.into();
            } else if let Ok(data) = serde_json::from_str::<T3>(&buf) {
                config = data.into();
            } else {
                T1::rename_file_old();
                config = T1::new();
            }
        } else {
            config = T1::new();
        }
        config.write();
        config
    }
}
