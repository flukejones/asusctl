use std::str::FromStr;
use std::sync::Arc;

use ::zbus::export::futures_util::lock::Mutex;
use config_traits::StdConfig;
use log::error;
use rog_platform::firmware_attributes::{
    AttrValue, Attribute, FirmwareAttribute, FirmwareAttributes
};
use rog_platform::platform::{RogPlatform, ThrottlePolicy};
use serde::{Deserialize, Serialize};
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Type, Value};
use zbus::{fdo, interface, Connection};

use crate::config::Config;
use crate::error::RogError;
use crate::ASUS_ZBUS_PATH;

const MOD_NAME: &str = "asus_armoury";

#[derive(Debug, Default, Clone, Deserialize, Serialize, Type, Value, OwnedValue)]
pub struct PossibleValues {
    strings: Vec<String>,
    nums: Vec<i32>
}

fn dbus_path_for_attr(attr_name: &str) -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!("{ASUS_ZBUS_PATH}/{MOD_NAME}/{attr_name}")).into()
}

pub struct AsusArmouryAttribute {
    attr: Attribute,
    config: Arc<Mutex<Config>>,
    /// platform control required here for access to PPD or Throttle profile
    platform: RogPlatform
}

impl AsusArmouryAttribute {
    pub fn new(attr: Attribute, platform: RogPlatform, config: Arc<Mutex<Config>>) -> Self {
        Self {
            attr,
            config,
            platform
        }
    }

    pub async fn start_tasks(self, connection: &Connection) -> Result<(), RogError> {
        // self.reload()
        //     .await
        //     .unwrap_or_else(|err| warn!("Controller error: {}", err));
        let path = dbus_path_for_attr(self.attr.name());
        connection
            .object_server()
            .at(path.clone(), self)
            .await
            .map_err(|e| error!("Couldn't add server at path: {path}, {e:?}"))
            .ok();
        Ok(())
    }
}

/// If return is `-1` on a property then there is avilable value for that
/// property
#[interface(name = "xyz.ljones.AsusArmoury")]
impl AsusArmouryAttribute {
    #[zbus(property)]
    fn name(&self) -> FirmwareAttribute {
        self.attr.name().into()
    }

    #[zbus(property)]
    async fn available_attrs(&self) -> Vec<String> {
        let mut attrs = Vec::new();
        if !matches!(self.attr.default_value(), AttrValue::None) {
            attrs.push("default_value".to_string());
        }
        if !matches!(self.attr.min_value(), AttrValue::None) {
            attrs.push("min_value".to_string());
        }
        if !matches!(self.attr.max_value(), AttrValue::None) {
            attrs.push("max_value".to_string());
        }
        if !matches!(self.attr.scalar_increment(), AttrValue::None) {
            attrs.push("scalar_increment".to_string());
        }
        if !matches!(self.attr.possible_values(), AttrValue::None) {
            attrs.push("possible_values".to_string());
        }
        // TODO: Don't unwrap, use error
        if let Ok(value) = self.attr.current_value().map_err(|e| {
            error!("Failed to read: {e:?}");
            e
        }) {
            if !matches!(value, AttrValue::None) {
                attrs.push("current_value".to_string());
            }
        }
        attrs
    }

    /// If return is `-1` then there is no default value
    #[zbus(property)]
    async fn default_value(&self) -> i32 {
        match self.attr.default_value() {
            AttrValue::Integer(i) => *i,
            _ => -1
        }
    }

    #[zbus(property)]
    async fn min_value(&self) -> i32 {
        match self.attr.min_value() {
            AttrValue::Integer(i) => *i,
            _ => -1
        }
    }

    #[zbus(property)]
    async fn max_value(&self) -> i32 {
        match self.attr.max_value() {
            AttrValue::Integer(i) => *i,
            _ => -1
        }
    }

    #[zbus(property)]
    async fn scalar_increment(&self) -> i32 {
        match self.attr.scalar_increment() {
            AttrValue::Integer(i) => *i,
            _ => -1
        }
    }

    #[zbus(property)]
    async fn possible_values(&self) -> Vec<i32> {
        match self.attr.possible_values() {
            AttrValue::EnumInt(i) => i.clone(),
            _ => Vec::default()
        }
    }

    #[zbus(property)]
    async fn current_value(&self) -> fdo::Result<i32> {
        if let Ok(AttrValue::Integer(i)) = self.attr.current_value() {
            return Ok(i);
        }
        Err(fdo::Error::Failed(
            "Could not read current value".to_string()
        ))
    }

    #[zbus(property)]
    async fn set_current_value(&mut self, value: i32) -> fdo::Result<()> {
        self.attr
            .set_current_value(AttrValue::Integer(value))
            .map_err(|e| {
                error!("Could not set value: {e:?}");
                e
            })?;
        let profile: ThrottlePolicy =
            ThrottlePolicy::from_str(self.platform.get_platform_profile()?.as_str())?;
        if let Some(tunings) = self.config.lock().await.tunings.get_mut(&profile) {
            if let Some(tune) = tunings.get_mut(&self.name()) {
                *tune = value;
            }
        }
        self.config.lock().await.write();
        Ok(())
    }
}

pub async fn start_attributes_zbus(
    server: &Connection,
    platform: RogPlatform,
    config: Arc<Mutex<Config>>
) -> Result<(), RogError> {
    for attr in FirmwareAttributes::new().attributes() {
        AsusArmouryAttribute::new(attr.clone(), platform.clone(), config.clone())
            .start_tasks(server)
            .await?;
    }
    Ok(())
}
