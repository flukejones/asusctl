pub static VERSION: &str = env!("CARGO_PKG_VERSION");

use rog_aura::{AuraModeNum, AuraZone};
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use zvariant_derive::Type;

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct SupportedFunctions {
    pub anime_ctrl: AnimeSupportedFunctions,
    pub charge_ctrl: ChargeSupportedFunctions,
    pub platform_profile: PlatformProfileFunctions,
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
pub struct PlatformProfileFunctions {
    pub platform_profile: bool,
    pub fan_curves: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct LedSupportedFunctions {
    pub prod_id: String,
    pub brightness_set: bool,
    pub stock_led_modes: Vec<AuraModeNum>,
    pub multizone_led_mode: Vec<AuraZone>,
    pub per_key_led_mode: bool,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct RogBiosSupportedFunctions {
    pub post_sound: bool,
    pub dedicated_gfx: bool,
    pub panel_overdrive: bool,
    pub dgpu_disable: bool,
    pub egpu_enable: bool,
}

impl fmt::Display for SupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.anime_ctrl)?;
        writeln!(f, "{}", self.charge_ctrl)?;
        writeln!(f, "{}", self.platform_profile)?;
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
impl fmt::Display for PlatformProfileFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Platform profiles:")?;
        writeln!(f, "\tplatform: {}", self.platform_profile)?;
        writeln!(f, "\tfan curves: {}", self.fan_curves)
    }
}
impl fmt::Display for LedSupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "LED:")?;
        writeln!(f, "\tBrightness control: {}", self.brightness_set)?;
        writeln!(f, "\tStock LED modes: {:?}", self.stock_led_modes)?;
        writeln!(f, "\tMultizone LED mode: {:?}", self.multizone_led_mode)?;
        writeln!(f, "\tPer key LED mode: {}", self.per_key_led_mode)
    }
}
impl fmt::Display for RogBiosSupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ROG BIOS:")?;
        writeln!(f, "\tPOST sound switch: {}", self.post_sound)?;
        writeln!(f, "\tPanel Overdrive: {}", self.panel_overdrive)?;
        writeln!(f, "\tdGPU disable switch: {}", self.dgpu_disable)?;
        writeln!(f, "\teGPU enable switch: {}", self.egpu_enable)?;
        writeln!(f, "\tDedicated GFX switch: {}", self.dedicated_gfx)?;
        Ok(())
    }
}
