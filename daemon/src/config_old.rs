use rog_fan_curve::Curve;
use rog_profiles::profiles::Profile;
use rog_types::gfx_vendors::GfxVendors;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::Config;

/// for parsing old v3.1.7 config
#[derive(Deserialize)]
pub(crate) struct ConfigV317 {
    pub gfx_mode: GfxVendors,
    pub gfx_managed: bool,
    pub active_profile: String,
    pub toggle_profiles: Vec<String>,
    #[serde(skip)]
    pub curr_fan_mode: u8,
    pub bat_charge_limit: u8,
    pub kbd_led_brightness: u8,
    pub kbd_backlight_mode: u8,
    #[serde(skip)]
    pub kbd_backlight_modes: Option<bool>,
    pub power_profiles: BTreeMap<String, ProfileV317>,
}

impl ConfigV317 {
    pub(crate) fn into_current(self) -> Config {
        Config {
            gfx_mode: GfxVendors::Hybrid,
            gfx_tmp_mode: None,
            gfx_managed: self.gfx_managed,
            gfx_vfio_enable: false,
            active_profile: self.active_profile,
            toggle_profiles: self.toggle_profiles,
            curr_fan_mode: self.curr_fan_mode,
            bat_charge_limit: self.bat_charge_limit,
            power_profiles: ProfileV317::transform_map(self.power_profiles),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ConfigV324 {
    pub gfx_mode: GfxVendors,
    pub gfx_managed: bool,
    pub active_profile: String,
    pub toggle_profiles: Vec<String>,
    #[serde(skip)]
    pub curr_fan_mode: u8,
    pub bat_charge_limit: u8,
    pub power_profiles: BTreeMap<String, ProfileV317>,
}

impl ConfigV324 {
    pub(crate) fn into_current(self) -> Config {
        Config {
            gfx_mode: GfxVendors::Hybrid,
            gfx_tmp_mode: None,
            gfx_managed: self.gfx_managed,
            gfx_vfio_enable: false,
            active_profile: self.active_profile,
            toggle_profiles: self.toggle_profiles,
            curr_fan_mode: self.curr_fan_mode,
            bat_charge_limit: self.bat_charge_limit,
            power_profiles: ProfileV317::transform_map(self.power_profiles),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ConfigV341 {
    pub gfx_mode: GfxVendors,
    pub gfx_managed: bool,
    pub gfx_vfio_enable: bool,
    pub active_profile: String,
    pub toggle_profiles: Vec<String>,
    #[serde(skip)]
    pub curr_fan_mode: u8,
    pub bat_charge_limit: u8,
    pub power_profiles: BTreeMap<String, ProfileV317>,
}

impl ConfigV341 {
    pub(crate) fn into_current(self) -> Config {
        Config {
            gfx_mode: GfxVendors::Hybrid,
            gfx_tmp_mode: None,
            gfx_managed: self.gfx_managed,
            gfx_vfio_enable: false,
            active_profile: self.active_profile,
            toggle_profiles: self.toggle_profiles,
            curr_fan_mode: self.curr_fan_mode,
            bat_charge_limit: self.bat_charge_limit,
            power_profiles: ProfileV317::transform_map(self.power_profiles),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ConfigV352 {
    pub gfx_mode: GfxVendors,
    pub gfx_last_mode: GfxVendors,
    pub gfx_managed: bool,
    pub gfx_vfio_enable: bool,
    pub gfx_save_compute_vfio: bool,
    pub active_profile: String,
    pub toggle_profiles: Vec<String>,
    #[serde(skip)]
    pub curr_fan_mode: u8,
    pub bat_charge_limit: u8,
    pub power_profiles: BTreeMap<String, ProfileV317>,
}

impl ConfigV352 {
    pub(crate) fn into_current(self) -> Config {
        Config {
            gfx_mode: GfxVendors::Hybrid,
            gfx_tmp_mode: None,
            gfx_managed: self.gfx_managed,
            gfx_vfio_enable: false,
            active_profile: self.active_profile,
            toggle_profiles: self.toggle_profiles,
            curr_fan_mode: self.curr_fan_mode,
            bat_charge_limit: self.bat_charge_limit,
            power_profiles: ProfileV317::transform_map(self.power_profiles),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileV317 {
    pub min_percentage: u8,
    pub max_percentage: u8,
    pub turbo: bool,
    pub fan_preset: u8,
    pub fan_curve: Option<Curve>,
}

impl ProfileV317 {
    fn into_current(self, name: String) -> Profile {
        Profile {
            name,
            min_percentage: self.min_percentage,
            max_percentage: self.max_percentage,
            turbo: self.turbo,
            fan_preset: self.fan_preset.into(),
            fan_curve: self
                .fan_curve
                .map_or_else(|| "".to_string(), |c| c.as_config_string()),
        }
    }

    fn transform_map(map: BTreeMap<String, ProfileV317>) -> BTreeMap<String, Profile> {
        let mut new_map = BTreeMap::new();
        map.iter().for_each(|(k, v)| {
            new_map.insert(k.to_string(), v.clone().into_current(k.to_string()));
        });
        new_map
    }
}
