use std::collections::BTreeMap;

use rog_aura::usb::{AuraDevRog2, AuraDevice, AuraPowerDev};
use rog_aura::{AuraEffect, AuraModeNum, AuraZone};
use rog_platform::platform::GpuMode;
use rog_platform::supported::{
    AdvancedAura, AnimeSupportedFunctions, ChargeSupportedFunctions, LedSupportedFunctions,
    PlatformProfileFunctions, RogBiosSupportedFunctions, SupportedFunctions,
};
use rog_profiles::fan_curve_set::{CurveData, FanCurveSet};
use supergfxctl::pci_device::{GfxMode, GfxPower};

use crate::error::Result;

const NOPE: &str = "";

#[derive(Default)]
pub struct DaemonProxyBlocking<'a> {
    _phantom: &'a str,
}

impl<'a> DaemonProxyBlocking<'a> {
    pub fn new(_c: &bool) -> Result<Self> {
        Ok(Self { _phantom: NOPE })
    }

    pub fn mode(&self) -> Result<GfxMode> {
        Ok(GfxMode::None)
    }

    pub fn power(&self) -> Result<GfxPower> {
        Ok(GfxPower::Suspended)
    }
}

#[derive(Default)]
pub struct RogDbusClientBlocking<'a> {
    _phantom: &'a str,
}

impl<'a> RogDbusClientBlocking<'a> {
    pub fn new() -> Result<(Self, bool)> {
        Ok((Self { _phantom: NOPE }, true))
    }

    pub fn proxies(&self) -> Proxies {
        Proxies
    }
}

pub struct Proxies;
impl Proxies {
    pub fn rog_bios(&self) -> Bios {
        Bios
    }

    pub fn profile(&self) -> Profile {
        Profile
    }

    pub fn led(&self) -> Led {
        Led
    }

    pub fn anime(&self) -> Anime {
        Anime
    }

    pub fn charge(&self) -> Profile {
        Profile
    }

    pub fn supported(&self) -> Supported {
        Supported
    }
}

pub struct Bios;
impl Bios {
    pub fn post_boot_sound(&self) -> Result<i16> {
        Ok(1)
    }

    pub fn gpu_mux_mode(&self) -> Result<GpuMode> {
        Ok(GpuMode::Optimus)
    }

    pub fn panel_od(&self) -> Result<bool> {
        Ok(true)
    }

    pub fn set_post_boot_sound(&self, _b: bool) -> Result<()> {
        Ok(())
    }

    pub fn set_gpu_mux_mode(&self, _b: GpuMode) -> Result<()> {
        Ok(())
    }

    pub fn set_panel_od(&self, _b: bool) -> Result<()> {
        Ok(())
    }
}

pub struct Profile;
impl Profile {
    pub fn profiles(&self) -> Result<Vec<rog_profiles::Profile>> {
        Ok(vec![
            rog_profiles::Profile::Balanced,
            rog_profiles::Profile::Performance,
            rog_profiles::Profile::Quiet,
        ])
    }

    pub fn active_profile(&self) -> Result<rog_profiles::Profile> {
        Ok(rog_profiles::Profile::Performance)
    }

    pub fn enabled_fan_profiles(&self) -> Result<Vec<rog_profiles::Profile>> {
        Ok(vec![
            rog_profiles::Profile::Performance,
            rog_profiles::Profile::Balanced,
        ])
    }

    pub fn fan_curve_data(&self, _p: rog_profiles::Profile) -> Result<FanCurveSet> {
        let mut curve = FanCurveSet::default();
        curve.cpu.pwm = [30, 40, 60, 100, 140, 180, 200, 250];
        curve.cpu.temp = [20, 30, 40, 50, 70, 80, 90, 100];
        curve.gpu.pwm = [40, 80, 100, 140, 170, 200, 230, 250];
        curve.gpu.temp = [20, 30, 40, 50, 70, 80, 90, 100];
        Ok(curve)
    }

    pub fn set_fan_curve(&self, _p: rog_profiles::Profile, _c: CurveData) -> Result<()> {
        Ok(())
    }

    pub fn set_fan_curve_enabled(&self, _p: rog_profiles::Profile, _b: bool) -> Result<()> {
        Ok(())
    }

    pub fn charge_control_end_threshold(&self) -> Result<u8> {
        Ok(66)
    }

    pub fn set_charge_control_end_threshold(&self, _l: u8) -> Result<()> {
        Ok(())
    }

    pub fn set_active_profile(&self, _p: rog_profiles::Profile) -> Result<()> {
        Ok(())
    }

    pub fn mains_online(&self) -> Result<bool> {
        Ok(true)
    }

    pub fn reset_profile_curves(&self, _p: rog_profiles::Profile) -> Result<bool> {
        Ok(true)
    }
}

pub struct Led;
impl Led {
    pub fn led_modes(&self) -> Result<BTreeMap<AuraModeNum, AuraEffect>> {
        let mut data = BTreeMap::new();
        data.insert(AuraModeNum::Static, AuraEffect::default());
        data.insert(AuraModeNum::Star, AuraEffect::default());
        data.insert(AuraModeNum::Strobe, AuraEffect::default());
        data.insert(AuraModeNum::Rain, AuraEffect::default());
        data.insert(AuraModeNum::Rainbow, AuraEffect::default());
        data.insert(AuraModeNum::Ripple, AuraEffect::default());
        data.insert(AuraModeNum::Breathe, AuraEffect::default());
        data.insert(AuraModeNum::Comet, AuraEffect::default());
        data.insert(AuraModeNum::Flash, AuraEffect::default());
        data.insert(AuraModeNum::Laser, AuraEffect::default());
        data.insert(AuraModeNum::Pulse, AuraEffect::default());
        Ok(data)
    }

    pub fn led_mode(&self) -> Result<AuraModeNum> {
        Ok(AuraModeNum::Rainbow)
    }

    pub fn led_brightness(&self) -> Result<i16> {
        Ok(1)
    }

    pub fn leds_enabled(&self) -> Result<AuraPowerDev> {
        Ok(AuraPowerDev {
            tuf: vec![],
            x1866: vec![],
            x19b6: vec![
                AuraDevRog2::BootKeyb,
                AuraDevRog2::AwakeKeyb,
                AuraDevRog2::SleepLogo,
                AuraDevRog2::AwakeLogo,
            ],
        })
    }

    pub fn set_leds_power(&self, _a: AuraPowerDev, _b: bool) -> Result<()> {
        Ok(())
    }

    pub fn set_led_mode(&self, _a: &AuraEffect) -> Result<()> {
        Ok(())
    }
}

pub struct Anime;
impl Anime {
    pub fn boot_enabled(&self) -> Result<bool> {
        Ok(true)
    }

    pub fn awake_enabled(&self) -> Result<bool> {
        Ok(true)
    }

    pub fn set_on_off(&self, _b: bool) -> Result<()> {
        Ok(())
    }

    pub fn set_boot_on_off(&self, _b: bool) -> Result<()> {
        Ok(())
    }
}

pub struct Supported;
impl Supported {
    pub fn supported_functions(&self) -> Result<SupportedFunctions> {
        Ok(SupportedFunctions {
            anime_ctrl: AnimeSupportedFunctions(true),
            charge_ctrl: ChargeSupportedFunctions {
                charge_level_set: true,
            },
            platform_profile: PlatformProfileFunctions {
                platform_profile: true,
                fan_curves: true,
            },
            keyboard_led: LedSupportedFunctions {
                dev_id: AuraDevice::X19b6,
                brightness: true,
                basic_modes: vec![
                    AuraModeNum::Rain,
                    AuraModeNum::Rainbow,
                    AuraModeNum::Star,
                    AuraModeNum::Static,
                    AuraModeNum::Strobe,
                ],
                basic_zones: vec![
                    AuraZone::Key1,
                    AuraZone::Key2,
                    AuraZone::Key3,
                    AuraZone::Key4,
                    AuraZone::BarLeft,
                    AuraZone::BarRight,
                    AuraZone::Logo,
                ],
                advanced_type: AdvancedAura::PerKey,
            },
            rog_bios_ctrl: RogBiosSupportedFunctions {
                post_sound: true,
                gpu_mux: true,
                panel_overdrive: true,
                dgpu_disable: true,
                mini_led_mode: true,
                egpu_enable: true,
            },
        })
    }
}
