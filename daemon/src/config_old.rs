use serde_derive::{Deserialize, Serialize};
use supergfxctl::gfx_vendors::GfxVendors;
use std::collections::BTreeMap;

use crate::config::Config;

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
            bat_charge_limit: self.bat_charge_limit,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ConfigV372 {
    pub gfx_mode: GfxVendors,
    /// Only for informational purposes.
    #[serde(skip)]
    pub gfx_tmp_mode: Option<GfxVendors>,
    pub gfx_managed: bool,
    pub gfx_vfio_enable: bool,
    pub active_profile: String,
    pub toggle_profiles: Vec<String>,
    #[serde(skip)]
    pub curr_fan_mode: u8,
    pub bat_charge_limit: u8,
    pub power_profiles: BTreeMap<String, ProfileV317>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileV317 {
    pub min_percentage: u8,
    pub max_percentage: u8,
    pub turbo: bool,
    pub fan_preset: u8,
    pub fan_curve: Option<()>,
}
