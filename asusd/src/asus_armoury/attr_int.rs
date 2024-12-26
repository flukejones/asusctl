use log::error;
use rog_platform::firmware_attributes::{AttrValue, Attribute};
use serde::{Deserialize, Serialize};
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Type, Value};
use zbus::{fdo, interface, Connection};

use crate::error::RogError;
use crate::ASUS_ZBUS_PATH;

const MOD_NAME: &str = "asus_armoury";

#[derive(Debug, Default, Clone, Deserialize, Serialize, Type, Value, OwnedValue)]
pub struct PossibleValues {
    strings: Vec<String>,
    nums: Vec<i32>,
}

fn dbus_path_for_attr(attr_name: &str) -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!("{ASUS_ZBUS_PATH}/{MOD_NAME}/{attr_name}")).into()
}

pub struct AsusArmouryAttribute(Attribute);

impl AsusArmouryAttribute {
    pub fn new(attr: Attribute) -> Self {
        Self(attr)
    }

    pub async fn start_tasks(self, connection: &Connection) -> Result<(), RogError> {
        // self.reload()
        //     .await
        //     .unwrap_or_else(|err| warn!("Controller error: {}", err));
        let path = dbus_path_for_attr(self.0.name());
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
#[interface(name = "org.asuslinux.AsusArmoury")]
impl AsusArmouryAttribute {
    #[zbus(property)]
    async fn name(&self) -> String {
        self.0.name().to_string()
    }

    /// If return is `-1` then there is no default value
    #[zbus(property)]
    async fn default_value(&self) -> i32 {
        match self.0.default_value() {
            AttrValue::Integer(i) => *i,
            _ => -1,
        }
    }

    #[zbus(property)]
    async fn min_value(&self) -> i32 {
        match self.0.min_value() {
            AttrValue::Integer(i) => *i,
            _ => -1,
        }
    }

    #[zbus(property)]
    async fn max_value(&self) -> i32 {
        match self.0.max_value() {
            AttrValue::Integer(i) => *i,
            _ => -1,
        }
    }

    #[zbus(property)]
    async fn scalar_increment(&self) -> i32 {
        self.0.scalar_increment().unwrap_or(1)
    }

    #[zbus(property)]
    async fn current_value(&self) -> fdo::Result<i32> {
        if let Ok(v) = self.0.current_value() {
            if let AttrValue::Integer(i) = v {
                return Ok(i);
            }
        }
        Err(fdo::Error::Failed(
            "Could not read current value".to_string(),
        ))
    }

    #[zbus(property)]
    async fn set_current_value(&mut self, value: i32) -> fdo::Result<()> {
        Ok(self
            .0
            .set_current_value(AttrValue::Integer(value))
            .map_err(|e| {
                error!("Could not set value: {e:?}");
                e
            })?)
    }
}
