use crate::slint_generatedMainWindow::{
    AuraPowerState as SlintAuraPowerState, LaptopAuraPower as SlintLaptopAuraPower,
};

impl From<rog_aura::AuraEffect> for crate::slint_generatedMainWindow::AuraEffect {
    fn from(m: rog_aura::AuraEffect) -> Self {
        Self {
            colour1: RgbaColor {
                red: m.colour1.r,
                green: m.colour1.g,
                blue: m.colour1.b,
                alpha: 255,
            }
            .into(),
            colour2: RgbaColor {
                red: m.colour2.r,
                green: m.colour2.g,
                blue: m.colour2.b,
                alpha: 255,
            }
            .into(),
            direction: m.direction.into(),
            mode: m.mode.into(),
            speed: m.speed.into(),
            zone: m.zone.into(),
        }
    }
}

impl From<crate::slint_generatedMainWindow::AuraEffect> for rog_aura::AuraEffect {
    fn from(m: crate::slint_generatedMainWindow::AuraEffect) -> Self {
        let c1: RgbaColor<u8> = m.colour1.into();
        let c2: RgbaColor<u8> = m.colour2.into();
        Self {
            colour1: rog_aura::Colour {
                r: c1.red,
                g: c1.green,
                b: c1.blue,
            },
            colour2: rog_aura::Colour {
                r: c2.red,
                g: c2.green,
                b: c2.blue,
            },
            direction: m.direction.into(),
            mode: m.mode.into(),
            speed: m.speed.into(),
            zone: m.zone.into(),
        }
    }
}

use rog_aura::aura_detection::PowerZones;
use rog_aura::keyboard::{AuraPowerState, LaptopAuraPower};
use slint::{Model, ModelRc, RgbaColor};

use crate::slint_generatedMainWindow::PowerZones as SlintPowerZones;
impl From<rog_aura::aura_detection::PowerZones> for SlintPowerZones {
    fn from(value: rog_aura::aura_detection::PowerZones) -> Self {
        match value {
            PowerZones::Logo => SlintPowerZones::Logo,
            PowerZones::Keyboard => SlintPowerZones::Keyboard,
            PowerZones::Lightbar => SlintPowerZones::Lightbar,
            PowerZones::Lid => SlintPowerZones::Lid,
            PowerZones::RearGlow => SlintPowerZones::RearGlow,
            PowerZones::KeyboardAndLightbar => todo!(),
        }
    }
}

impl From<SlintPowerZones> for rog_aura::aura_detection::PowerZones {
    fn from(value: SlintPowerZones) -> Self {
        match value {
            SlintPowerZones::Logo => rog_aura::aura_detection::PowerZones::Logo,
            SlintPowerZones::Keyboard => rog_aura::aura_detection::PowerZones::Keyboard,
            SlintPowerZones::Lightbar => rog_aura::aura_detection::PowerZones::Lightbar,
            SlintPowerZones::Lid => rog_aura::aura_detection::PowerZones::Lid,
            SlintPowerZones::RearGlow => rog_aura::aura_detection::PowerZones::RearGlow,
            SlintPowerZones::KeyboardAndLightbar => {
                rog_aura::aura_detection::PowerZones::KeyboardAndLightbar
            }
        }
    }
}

impl From<SlintAuraPowerState> for AuraPowerState {
    fn from(value: SlintAuraPowerState) -> Self {
        Self {
            zone: value.zone.into(),
            boot: value.boot.into(),
            awake: value.awake.into(),
            sleep: value.sleep.into(),
            shutdown: value.shutdown.into(),
        }
    }
}

impl From<AuraPowerState> for SlintAuraPowerState {
    fn from(value: AuraPowerState) -> Self {
        let zone = value.zone.into();
        SlintAuraPowerState {
            boot: value.boot.into(),
            awake: value.awake.into(),
            sleep: value.sleep.into(),
            shutdown: value.shutdown.into(),
            zone,
            zone_name_idx: zone as i32,
        }
    }
}

impl From<&AuraPowerState> for SlintAuraPowerState {
    fn from(value: &AuraPowerState) -> Self {
        let zone = value.zone.into();
        SlintAuraPowerState {
            boot: value.boot.into(),
            awake: value.awake.into(),
            sleep: value.sleep.into(),
            shutdown: value.shutdown.into(),
            zone,
            zone_name_idx: zone as i32,
        }
    }
}

impl From<SlintLaptopAuraPower> for rog_aura::keyboard::LaptopAuraPower {
    fn from(value: SlintLaptopAuraPower) -> Self {
        let mut states = Vec::new();
        for state in value.states.iter() {
            states.push(state.into());
        }
        Self { states }
    }
}

impl From<LaptopAuraPower> for SlintLaptopAuraPower {
    fn from(value: LaptopAuraPower) -> Self {
        let converted: Vec<SlintAuraPowerState> = value
            .states
            .iter()
            .map(|s| SlintAuraPowerState::from(s))
            .collect();
        Self {
            states: ModelRc::from(converted.as_slice()),
        }
    }
}
