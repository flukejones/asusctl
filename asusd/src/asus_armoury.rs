use std::sync::Arc;

use ::zbus::export::futures_util::lock::Mutex;
use config_traits::StdConfig;
use log::{debug, error, info};
use rog_platform::asus_armoury::{AttrValue, Attribute, FirmwareAttribute, FirmwareAttributes};
use rog_platform::platform::{PlatformProfile, RogPlatform};
use rog_platform::power::AsusPower;
use serde::{Deserialize, Serialize};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Type, Value};
use zbus::{fdo, interface, Connection};

use crate::config::Config;
use crate::error::RogError;
use crate::{Reloadable, ASUS_ZBUS_PATH};

const MOD_NAME: &str = "asus_armoury";

#[derive(Debug, Default, Clone, Deserialize, Serialize, Type, Value, OwnedValue)]
pub struct PossibleValues {
    strings: Vec<String>,
    nums: Vec<i32>
}

fn dbus_path_for_attr(attr_name: &str) -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!("{ASUS_ZBUS_PATH}/{MOD_NAME}/{attr_name}")).into()
}

#[derive(Clone)]
pub struct AsusArmouryAttribute {
    attr: Attribute,
    config: Arc<Mutex<Config>>,
    /// platform control required here for access to PPD or Throttle profile
    platform: RogPlatform,
    power: AsusPower
}

impl AsusArmouryAttribute {
    pub fn new(
        attr: Attribute,
        platform: RogPlatform,
        power: AsusPower,
        config: Arc<Mutex<Config>>
    ) -> Self {
        Self {
            attr,
            config,
            platform,
            power
        }
    }

    pub async fn move_to_zbus(self, connection: &Connection) -> Result<(), RogError> {
        let path = dbus_path_for_attr(self.attr.name());
        connection
            .object_server()
            .at(path.clone(), self)
            .await
            .map_err(|e| error!("Couldn't add server at path: {path}, {e:?}"))
            .ok();
        Ok(())
    }

    async fn watch_and_notify(
        &mut self,
        signal_ctxt: SignalEmitter<'static>
    ) -> Result<(), RogError> {
        use zbus::export::futures_util::StreamExt;

        let name = self.name();
        macro_rules! watch_value_notify {
            ($attr_str:expr, $fn_prop_changed:ident) => {
                match self.attr.get_watcher($attr_str) {
                    Ok(watch) => {
                        let name = <&str>::from(name);
                        let ctrl = self.clone();
                        let sig = signal_ctxt.clone();
                        tokio::spawn(async move {
                            let mut buffer = [0; 32];
                            watch
                                .into_event_stream(&mut buffer)
                                .unwrap()
                                .for_each(|_| async {
                                    debug!("{} changed", name);
                                    ctrl.$fn_prop_changed(&sig).await.ok();
                                })
                                .await;
                        });
                    }
                    Err(e) => info!(
                        "inotify watch failed: {}. You can ignore this if your device does not \
                         support the feature",
                        e
                    )
                }
            };
        }

        // "current_value", "default_value", "min_value", "max_value"
        watch_value_notify!("current_value", current_value_changed);
        watch_value_notify!("default_value", default_value_changed);
        watch_value_notify!("min_value", min_value_changed);
        watch_value_notify!("max_value", max_value_changed);

        Ok(())
    }
}

impl crate::Reloadable for AsusArmouryAttribute {
    async fn reload(&mut self) -> Result<(), RogError> {
        info!("Reloading {}", self.attr.name());
        let profile: PlatformProfile = self.platform.get_platform_profile()?.into();
        let power_plugged = self
            .power
            .get_online()
            .map_err(|e| {
                error!("Could not get power status: {e:?}");
                e
            })
            .unwrap_or_default();
        let config = if power_plugged == 1 {
            &self.config.lock().await.ac_profile_tunings
        } else {
            &self.config.lock().await.dc_profile_tunings
        };
        if let Some(tuning) = config.get(&profile) {
            if tuning.enabled {
                if let Some(tune) = tuning.group.get(&self.name()) {
                    self.attr
                        .set_current_value(&AttrValue::Integer(*tune))
                        .map_err(|e| {
                            error!("Could not set {} value: {e:?}", self.attr.name());
                            self.attr.base_path_exists();
                            e
                        })?;
                    info!("Set {} to {:?}", self.attr.name(), tune);
                }
            }
        }

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

    async fn restore_default(&self) -> fdo::Result<()> {
        self.attr.restore_default()?;
        if self.name().is_ppt() {
            let profile: PlatformProfile = self.platform.get_platform_profile()?.into();
            let power_plugged = self
                .power
                .get_online()
                .map_err(|e| {
                    error!("Could not get power status: {e:?}");
                    e
                })
                .unwrap_or_default();

            let mut config = self.config.lock().await;
            let tuning = config.select_tunings(power_plugged == 1, profile);
            if let Some(tune) = tuning.group.get_mut(&self.name()) {
                if let AttrValue::Integer(i) = self.attr.default_value() {
                    *tune = *i;
                }
            }
            if tuning.enabled {
                self.attr
                    .set_current_value(self.attr.default_value())
                    .map_err(|e| {
                        error!("Could not set value: {e:?}");
                        e
                    })?;
            }
            config.write();
        }
        Ok(())
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
        if self.name().is_ppt() {
            let profile: PlatformProfile = self.platform.get_platform_profile()?.into();
            let power_plugged = self
                .power
                .get_online()
                .map_err(|e| {
                    error!("Could not get power status: {e:?}");
                    e
                })
                .unwrap_or_default();
            let mut config = self.config.lock().await;
            let tuning = config.select_tunings(power_plugged == 1, profile);
            if let Some(tune) = tuning.group.get(&self.name()) {
                return Ok(*tune);
            } else if let AttrValue::Integer(i) = self.attr.default_value() {
                return Ok(*i);
            }
            return Err(fdo::Error::Failed(
                "Could not read current value".to_string()
            ));
        }

        if let Ok(AttrValue::Integer(i)) = self.attr.current_value() {
            return Ok(i);
        }
        Err(fdo::Error::Failed(
            "Could not read current value".to_string()
        ))
    }

    #[zbus(property)]
    async fn set_current_value(&mut self, value: i32) -> fdo::Result<()> {
        if self.name().is_ppt() {
            let profile: PlatformProfile = self.platform.get_platform_profile()?.into();
            let power_plugged = self
                .power
                .get_online()
                .map_err(|e| {
                    error!("Could not get power status: {e:?}");
                    e
                })
                .unwrap_or_default();

            let mut config = self.config.lock().await;
            let tuning = config.select_tunings(power_plugged == 1, profile);

            if let Some(tune) = tuning.group.get_mut(&self.name()) {
                *tune = value;
            } else {
                tuning.group.insert(self.name(), value);
                debug!("Store tuning config for {} = {:?}", self.attr.name(), value);
            }
            if tuning.enabled {
                self.attr
                    .set_current_value(&AttrValue::Integer(value))
                    .map_err(|e| {
                        error!("Could not set value: {e:?}");
                        e
                    })?;
            }
        } else {
            self.attr
                .set_current_value(&AttrValue::Integer(value))
                .map_err(|e| {
                    error!("Could not set value: {e:?}");
                    e
                })?;

            let has_attr = self
                .config
                .lock()
                .await
                .armoury_settings
                .contains_key(&self.name());
            if has_attr {
                if let Some(setting) = self
                    .config
                    .lock()
                    .await
                    .armoury_settings
                    .get_mut(&self.name())
                {
                    *setting = value
                }
            } else {
                debug!("Adding config for {}", self.attr.name());
                self.config
                    .lock()
                    .await
                    .armoury_settings
                    .insert(self.name(), value);
                debug!("Set config for {} = {:?}", self.attr.name(), value);
            }
        }
        self.config.lock().await.write();
        Ok(())
    }
}

pub async fn start_attributes_zbus(
    conn: &Connection,
    platform: RogPlatform,
    power: AsusPower,
    attributes: FirmwareAttributes,
    config: Arc<Mutex<Config>>
) -> Result<(), RogError> {
    for attr in attributes.attributes() {
        let mut attr = AsusArmouryAttribute::new(
            attr.clone(),
            platform.clone(),
            power.clone(),
            config.clone()
        );
        attr.reload().await?;

        let path = dbus_path_for_attr(attr.attr.name());
        let sig = zbus::object_server::SignalEmitter::new(conn, path)?;
        attr.watch_and_notify(sig).await?;

        attr.move_to_zbus(conn).await?;
    }
    Ok(())
}

pub async fn set_config_or_default(
    attrs: &FirmwareAttributes,
    config: &mut Config,
    power_plugged: bool,
    profile: PlatformProfile
) {
    for attr in attrs.attributes().iter() {
        let name: FirmwareAttribute = attr.name().into();
        if name.is_ppt() {
            let tuning = config.select_tunings(power_plugged, profile);
            if !tuning.enabled {
                debug!("Tuning group is not enabled, skipping");
                return;
            }

            if let Some(tune) = tuning.group.get(&name) {
                attr.set_current_value(&AttrValue::Integer(*tune))
                    .map_err(|e| {
                        error!("Failed to set {}: {e}", <&str>::from(name));
                    })
                    .ok();
            } else {
                let default = attr.default_value();
                attr.set_current_value(default)
                    .map_err(|e| {
                        error!("Failed to set {}: {e}", <&str>::from(name));
                    })
                    .ok();
                if let AttrValue::Integer(i) = default {
                    tuning.group.insert(name, *i);
                    info!(
                        "Set default tuning config for {} = {:?}",
                        <&str>::from(name),
                        i
                    );
                    // config.write();
                }
            }
        }
    }
}
