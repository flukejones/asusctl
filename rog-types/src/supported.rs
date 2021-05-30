use rog_aura::AuraModeNum;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use zvariant_derive::Type;

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct SupportedFunctions {
    pub anime_ctrl: AnimeSupportedFunctions,
    pub charge_ctrl: ChargeSupportedFunctions,
    pub fan_cpu_ctrl: FanCpuSupportedFunctions,
    pub keyboard_led: LedSupportedFunctions,
    pub rog_bios_ctrl: RogBiosSupportedFunctions,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct AnimeSupportedFunctions(pub bool);

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct ChargeSupportedFunctions {
    pub charge_level_set: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct FanCpuSupportedFunctions {
    pub stock_fan_modes: bool,
    pub min_max_freq: bool,
    pub fan_curve_set: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct LedSupportedFunctions {
    pub brightness_set: bool,
    pub stock_led_modes: Vec<AuraModeNum>,
    pub multizone_led_mode: bool,
    pub per_key_led_mode: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RogBiosSupportedFunctions {
    pub post_sound_toggle: bool,
    pub dedicated_gfx_toggle: bool,
}

impl fmt::Display for SupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.anime_ctrl)?;
        writeln!(f, "{}", self.charge_ctrl)?;
        writeln!(f, "{}", self.fan_cpu_ctrl)?;
        writeln!(f, "{}", self.keyboard_led)?;
        writeln!(f, "{}", self.rog_bios_ctrl)
    }
}

impl fmt::Display for AnimeSupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "AniMe Matrix:")?;
        writeln!(f, "\tAnime Matrix control: {}", self.0)
    }
}
impl fmt::Display for ChargeSupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Charge:")?;
        writeln!(
            f,
            "\tBattery charge limit control: {}",
            self.charge_level_set
        )
    }
}
impl fmt::Display for FanCpuSupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Fan:")?;
        writeln!(f, "\tStock fan modes: {}", self.stock_fan_modes)?;
        writeln!(f, "\tMin/max frequency: {}", self.min_max_freq)?;
        writeln!(f, "\tFan curve control: {}", self.fan_curve_set)
    }
}
impl fmt::Display for LedSupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "LED:")?;
        writeln!(f, "\tBrightness control: {}", self.brightness_set)?;
        writeln!(f, "\tStock LED modes: {:?}", self.stock_led_modes)?;
        writeln!(f, "\tMultizone LED mode: {}", self.multizone_led_mode)?;
        writeln!(f, "\tPer key LED mode: {}", self.per_key_led_mode)
    }
}
impl fmt::Display for RogBiosSupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ROG BIOS:")?;
        writeln!(f, "\tPOST sound toggle: {}", self.post_sound_toggle)?;
        writeln!(f, "\tDedicated GFX toggle: {}", self.dedicated_gfx_toggle)
    }
}
