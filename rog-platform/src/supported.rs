use std::fmt;

use rog_aura::aura_detection::PowerZones;
use rog_aura::usb::AuraDevice;
use rog_aura::{AdvancedAuraType, AuraModeNum, AuraZone};
use rog_profiles::FanCurvePU;
use serde_derive::{Deserialize, Serialize};
use typeshare::typeshare;
use zbus::zvariant::Type;

#[typeshare]
#[derive(Serialize, Deserialize, Type, Debug, Default, Clone)]
pub struct SupportedFunctions {
    pub anime_ctrl: AnimeSupportedFunctions,
    pub charge_ctrl: ChargeSupportedFunctions,
    pub platform_profile: PlatformProfileFunctions,
    pub keyboard_led: LedSupportedFunctions,
    pub rog_bios_ctrl: PlatformSupportedFunctions,
}

#[typeshare]
#[derive(Serialize, Deserialize, Type, Debug, Default, Clone)]
pub struct AnimeSupportedFunctions(pub bool);

#[typeshare]
#[derive(Serialize, Deserialize, Type, Debug, Default, Clone)]
pub struct ChargeSupportedFunctions {
    pub charge_level_set: bool,
}

#[typeshare]
#[derive(Serialize, Deserialize, Type, Debug, Default, Clone)]
pub struct PlatformProfileFunctions {
    pub platform_profile: bool,
    pub fans: Vec<FanCurvePU>,
}

#[typeshare]
#[derive(Serialize, Deserialize, Default, Type, Debug, Clone)]
#[zvariant(signature = "s")]
pub enum AdvancedAura {
    #[default]
    None,
    Zoned,
    PerKey,
}

impl From<AdvancedAuraType> for AdvancedAura {
    fn from(a: AdvancedAuraType) -> Self {
        match a {
            AdvancedAuraType::None => Self::None,
            AdvancedAuraType::Zoned(_) => Self::Zoned,
            AdvancedAuraType::PerKey => Self::PerKey,
        }
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Type, Debug, Default, Clone)]
pub struct LedSupportedFunctions {
    pub dev_id: AuraDevice,
    pub brightness: bool,
    pub basic_modes: Vec<AuraModeNum>,
    pub basic_zones: Vec<AuraZone>,
    pub advanced_type: AdvancedAura,
    pub power_zones: Vec<PowerZones>,
}

#[typeshare]
#[derive(Serialize, Deserialize, Type, Debug, Default, Clone)]
pub struct PlatformSupportedFunctions {
    pub post_animation_sound: bool,
    pub gpu_mux: bool,
    pub panel_overdrive: bool,
    pub dgpu_disable: bool,
    pub egpu_enable: bool,
    pub mini_led_mode: bool,

    pub ppt_pl1_spl: bool,
    pub ppt_pl2_sppt: bool,
    pub ppt_fppt: bool,
    pub ppt_apu_sppt: bool,
    pub ppt_platform_sppt: bool,
    pub nv_dynamic_boost: bool,
    pub nv_temp_target: bool,
}

impl fmt::Display for SupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{}", self.anime_ctrl)?;
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
        writeln!(f, "\tfan curves: {:?}", self.fans)
    }
}
impl fmt::Display for LedSupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "LED:")?;
        writeln!(f, "\tDevice ID: {:?}", self.dev_id)?;
        writeln!(f, "\tBrightness control: {}", self.brightness)?;
        writeln!(f, "\tBasic modes: {:?}", self.basic_modes)?;
        writeln!(f, "\tBasic zones: {:?}", self.basic_zones)?;
        writeln!(f, "\tAdvanced modes: {:?}", self.advanced_type)
    }
}

impl fmt::Display for PlatformSupportedFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ROG BIOS:")?;
        writeln!(f, "\tPOST sound switch: {}", self.post_animation_sound)?;
        writeln!(f, "\tPanel Overdrive: {}", self.panel_overdrive)?;
        writeln!(f, "\tMiniLED backlight: {}", self.mini_led_mode)?;
        writeln!(f, "\tdGPU disable switch: {}", self.dgpu_disable)?;
        writeln!(f, "\teGPU enable switch: {}", self.egpu_enable)?;
        writeln!(f, "\tGPU MUX control: {}", self.gpu_mux)?;
        writeln!(f, "\tppt_pl1_spl: {}", self.ppt_pl1_spl)?;
        writeln!(f, "\tppt_pl2_sppt: {}", self.ppt_pl2_sppt)?;
        writeln!(f, "\tppt_fppt {}", self.ppt_fppt)?;
        writeln!(f, "\tppt_apu_sppt: {}", self.ppt_apu_sppt)?;
        writeln!(f, "\tppt_platform_sppt: {}", self.ppt_platform_sppt)?;
        writeln!(f, "\tnv_dynamic_boost: {}", self.nv_dynamic_boost)?;
        writeln!(f, "\tnv_temp_target: {}", self.nv_temp_target)?;
        Ok(())
    }
}
