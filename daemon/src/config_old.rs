use rog_aura::AuraEffect;
use rog_types::{gfx_vendors::GfxVendors, profile::Profile};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::Config;

/// for parsing old v3.1.7 config
#[derive(Deserialize, Serialize)]
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
    pub power_profiles: BTreeMap<String, Profile>,
}

impl ConfigV317 {
    pub(crate) fn into_current(self) -> Config {
        Config {
            gfx_mode: GfxVendors::Hybrid,
            gfx_last_mode: GfxVendors::Hybrid,
            gfx_managed: self.gfx_managed,
            gfx_vfio_enable: false,
            active_profile: self.active_profile,
            toggle_profiles: self.toggle_profiles,
            curr_fan_mode: self.curr_fan_mode,
            bat_charge_limit: self.bat_charge_limit,
            power_profiles: self.power_profiles,
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
    pub power_profiles: BTreeMap<String, Profile>,
}

impl ConfigV324 {
    pub(crate) fn into_current(self) -> Config {
        Config {
            gfx_mode: GfxVendors::Hybrid,
            gfx_last_mode: GfxVendors::Hybrid,
            gfx_managed: self.gfx_managed,
            gfx_vfio_enable: false,
            active_profile: self.active_profile,
            toggle_profiles: self.toggle_profiles,
            curr_fan_mode: self.curr_fan_mode,
            bat_charge_limit: self.bat_charge_limit,
            power_profiles: self.power_profiles,
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
    pub power_profiles: BTreeMap<String, Profile>,
}

impl ConfigV341 {
    pub(crate) fn into_current(self) -> Config {
        Config {
            gfx_mode: GfxVendors::Hybrid,
            gfx_last_mode: GfxVendors::Hybrid,
            gfx_managed: self.gfx_managed,
            gfx_vfio_enable: false,
            active_profile: self.active_profile,
            toggle_profiles: self.toggle_profiles,
            curr_fan_mode: self.curr_fan_mode,
            bat_charge_limit: self.bat_charge_limit,
            power_profiles: self.power_profiles,
        }
    }
}