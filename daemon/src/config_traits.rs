use std::fs::{create_dir, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use log::{error, warn};
use ron::ser::PrettyConfig;
use serde::de::DeserializeOwned;
use serde::Serialize;

static CONFIG_PATH_BASE: &str = "/etc/asusd/";

/// Create a `PathBuf` for `file`. If the base config dir `CONFIG_PATH_BASE`
/// does not exist it is created.
fn config_file(file: &str) -> PathBuf {
    let mut config = PathBuf::from(CONFIG_PATH_BASE);
    if !config.exists() {
        create_dir(config.as_path()).unwrap_or_else(|_| panic!("Could not create {config:?}"));
    }
    config.push(file);
    config
}

/// Open a config file as read/write. If the file or dir does not exist then
/// both are created.
fn config_file_open(file: PathBuf) -> File {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file.clone())
        .unwrap_or_else(|_| panic!("The file {file:?} or directory {CONFIG_PATH_BASE} is missing"))
}

pub trait StdConfig
where
    Self: Serialize + DeserializeOwned,
{
    fn new() -> Self;

    fn file_name() -> &'static str;

    fn file_path() -> PathBuf {
        config_file(Self::file_name())
    }

    fn file_open() -> File {
        config_file_open(Self::file_path())
    }

    fn read(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .open(Self::file_path())
            .unwrap_or_else(|err| panic!("Error reading {:?}: {}", Self::file_path(), err));
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
        let mut file = File::create(Self::file_path()).expect(&format!(
            "Couldn't overwrite config {:?}",
            Self::file_path()
        ));
        let ron = ron::ser::to_string_pretty(&self, PrettyConfig::new().depth_limit(4))
            .expect("Parse config to RON failed");
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
            panic!(
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
