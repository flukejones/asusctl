use std::fs::{create_dir, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use log::{error, warn};
pub use ron;
use ron::ser::PrettyConfig;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Config file helper traits. Only `new()` and `file_name()` are required to be
/// implemented, the rest are intended to be free methods.
pub trait StdConfig
where
    Self: Serialize + DeserializeOwned,
{
    /// Taking over the standard `new()` to ensure things can be generic
    fn new() -> Self;

    /// Return the config files names, such as `wibble.cfg`
    fn file_name(&self) -> String;

    /// Return the full path to the directory the config file resides in
    fn config_dir() -> PathBuf;

    /// Return the full path to the config file
    fn file_path(&self) -> PathBuf {
        let mut config = Self::config_dir();
        if !config.exists() {
            create_dir(config.as_path())
                .unwrap_or_else(|e| panic!("Could not create {:?} {e}", Self::config_dir()));
        }
        config.push(self.file_name());
        config
    }

    /// Directly open the config file for read and write. If the config file
    /// does not exist it is created, including the directories the file
    /// resides in.
    fn file_open(&self) -> File {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.file_path())
            .unwrap_or_else(|e| panic!("Could not open {:?} {e}", self.file_path()))
    }

    /// Open and parse the config file to self from ron format
    fn read(&mut self) {
        let mut file = match OpenOptions::new().read(true).open(self.file_path()) {
            Ok(data) => data,
            Err(err) => {
                error!("Error reading {:?}: {}", self.file_path(), err);
                return;
            }
        };
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                warn!("File is empty {:?}", self.file_path());
            } else if let Ok(data) = ron::from_str(&buf) {
                *self = data;
            } else {
                warn!("Could not deserialise {:?}", self.file_path());
            }
        }
    }

    /// Write the config file data to pretty ron format
    fn write(&self) {
        let mut file = match File::create(self.file_path()) {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Couldn't overwrite config {:?}, error: {e}",
                    self.file_path()
                );
                return;
            }
        };
        let ron = match ron::ser::to_string_pretty(&self, PrettyConfig::new().depth_limit(4)) {
            Ok(data) => data,
            Err(e) => {
                error!("Parse {:?} to RON failed, error: {e}", self.file_path());
                return;
            }
        };
        file.write_all(ron.as_bytes())
            .unwrap_or_else(|err| error!("Could not write config: {}", err));
    }

    /// Renames the existing file to `<file>-old`
    fn rename_file_old(&self) {
        warn!(
            "Renaming {} to {}-old and recreating config",
            self.file_name(),
            self.file_name()
        );
        let cfg_old = self.file_path().to_string_lossy().to_string() + "-old";
        std::fs::rename(self.file_path(), cfg_old).unwrap_or_else(|err| {
            error!(
                "Could not rename. Please remove {} then restart service: Error {}",
                self.file_name(),
                err
            )
        });
    }
}

/// Base trait for loading/parsing. This can be used to help update configs to
/// new versions
///
/// # Example
/// ```rust
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use config_traits::{StdConfig, StdConfigLoad1};
///
/// #[derive(Deserialize, Serialize)]
/// struct FanCurveConfig {}
///
/// impl StdConfig for FanCurveConfig {
///     fn new() -> Self { Self {} }
///
///     fn file_name(&self) -> std::string::String { "test_name.conf".to_owned() }
///
///     fn config_dir() -> PathBuf { PathBuf::from("/tmp") }
/// }
///
/// impl StdConfigLoad1 for FanCurveConfig {}
/// ```
///
/// If all of the generics fails to parse, then the old config is renamed and a
/// new one created
pub trait StdConfigLoad1
where
    Self: StdConfig + DeserializeOwned + Serialize,
{
    fn load(mut self) -> Self {
        let mut file = self.file_open();
        let mut buf = String::new();
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len != 0 {
                if let Ok(data) = ron::from_str(&buf) {
                    self = data;
                } else if let Ok(data) = serde_json::from_str(&buf) {
                    self = data;
                } else {
                    self.rename_file_old();
                    self = Self::new();
                }
            } else {
                error!("Config file {} zero read length", self.file_name());
            }
        }
        self.write();
        self
    }
}

/// Base trait for loading/parsing. This is intended to be used to help update
/// configs to new versions
///
/// # Example
/// ```rust
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use config_traits::{StdConfig, StdConfigLoad2};
///
/// #[derive(Deserialize, Serialize)]
/// struct FanCurveConfigOld {}
///
/// #[derive(Deserialize, Serialize)]
/// struct FanCurveConfig {}
///
/// impl From<FanCurveConfigOld> for FanCurveConfig {
///     fn from(_: FanCurveConfigOld) -> Self { Self {} }
/// }
///
/// impl StdConfig for FanCurveConfig {
///     fn new() -> Self { Self {} }
///
///     fn file_name(&self) -> std::string::String { "test_name.conf".to_owned() }
///
///     fn config_dir() -> PathBuf { PathBuf::from("/tmp") }
/// }
///
/// impl StdConfigLoad2<FanCurveConfigOld> for FanCurveConfig {}
/// ```
///
/// If all of the generics fails to parse, then the old config is renamed and a
/// new one created
pub trait StdConfigLoad2<OldConfig>
where
    Self: StdConfig + DeserializeOwned + Serialize,
    OldConfig: DeserializeOwned + Into<Self>,
{
    fn load(mut self) -> Self {
        let mut file = self.file_open();
        let mut buf = String::new();
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len != 0 {
                if let Ok(data) = ron::from_str(&buf) {
                    self = data;
                } else if let Ok(data) = serde_json::from_str(&buf) {
                    self = data;
                } else if let Ok(data) = serde_json::from_str::<OldConfig>(&buf) {
                    self = data.into();
                } else {
                    self.rename_file_old();
                    self = Self::new();
                }
            } else {
                error!("Config file {} zero read length", self.file_name());
            }
        }
        self.write();
        self
    }
}

/// Base trait for loading/parsing. This is intended to be used to help update
/// configs to new versions
///
/// # Example
/// ```rust
/// use std::path::PathBuf;
/// use serde::{Deserialize, Serialize};
/// use config_traits::{StdConfig, StdConfigLoad3};
///
/// #[derive(Deserialize, Serialize)]
/// struct FanCurveConfigOld {}
///
/// #[derive(Deserialize, Serialize)]
/// struct FanCurveConfigOlder {}
///
/// #[derive(Deserialize, Serialize)]
/// struct FanCurveConfig {}
///
/// impl From<FanCurveConfigOld> for FanCurveConfig {
///     fn from(_: FanCurveConfigOld) -> Self { Self {} }
/// }
///
/// impl From<FanCurveConfigOlder> for FanCurveConfig {
///     fn from(_: FanCurveConfigOlder) -> Self { Self {} }
/// }
///
/// impl StdConfig for FanCurveConfig {
///     fn new() -> Self { Self {} }
///
///     fn file_name(&self) -> std::string::String { "test_name.conf".to_owned() }
///
///     fn config_dir() -> PathBuf { PathBuf::from("/tmp") }
/// }
///
/// impl StdConfigLoad3<FanCurveConfigOld, FanCurveConfigOlder> for FanCurveConfig {}
/// ```
///
/// If all of the generics fails to parse, then the old config is renamed and a
/// new one created
pub trait StdConfigLoad3<OldConfig, OldConfig2>: StdConfig
where
    Self: StdConfig + DeserializeOwned + Serialize,
    OldConfig: DeserializeOwned + Into<Self>,
    OldConfig2: DeserializeOwned + Into<Self>,
{
    fn load(mut self) -> Self {
        let mut file = self.file_open();
        let mut buf = String::new();
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len != 0 {
                if let Ok(data) = ron::from_str(&buf) {
                    self = data;
                } else if let Ok(data) = serde_json::from_str(&buf) {
                    self = data;
                } else if let Ok(data) = serde_json::from_str::<OldConfig>(&buf) {
                    self = data.into();
                } else if let Ok(data) = serde_json::from_str::<OldConfig2>(&buf) {
                    self = data.into();
                } else {
                    self.rename_file_old();
                    self = Self::new();
                }
            } else {
                error!("Config file {} zero read length", self.file_name());
            }
        }
        self.write();
        self
    }
}
