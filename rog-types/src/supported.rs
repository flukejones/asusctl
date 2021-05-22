use rog_aura::AuraModeNum;
use serde_derive::{Deserialize, Serialize};
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
