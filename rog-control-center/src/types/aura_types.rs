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

use rog_aura::keyboard::{AuraPowerState, LaptopAuraPower};
use rog_aura::PowerZones;
use slint::{Model, ModelRc, RgbaColor};

use crate::slint_generatedMainWindow::PowerZones as SlintPowerZones;
impl From<PowerZones> for SlintPowerZones {
    fn from(value: PowerZones) -> Self {
        match value {
            PowerZones::Logo => SlintPowerZones::Logo,
            PowerZones::Keyboard => SlintPowerZones::Keyboard,
            PowerZones::Lightbar => SlintPowerZones::Lightbar,
            PowerZones::Lid => SlintPowerZones::Lid,
            PowerZones::RearGlow => SlintPowerZones::RearGlow,
            PowerZones::KeyboardAndLightbar => todo!(),
            PowerZones::None => todo!(),
        }
    }
}

impl From<SlintPowerZones> for PowerZones {
    fn from(value: SlintPowerZones) -> Self {
        match value {
            SlintPowerZones::Logo => PowerZones::Logo,
            SlintPowerZones::Keyboard => PowerZones::Keyboard,
            SlintPowerZones::Lightbar => PowerZones::Lightbar,
            SlintPowerZones::Lid => PowerZones::Lid,
            SlintPowerZones::RearGlow => PowerZones::RearGlow,
            SlintPowerZones::KeyboardAndLightbar => PowerZones::KeyboardAndLightbar,
        }
    }
}

impl From<SlintAuraPowerState> for AuraPowerState {
    fn from(value: SlintAuraPowerState) -> Self {
        Self {
            zone: value.zone.into(),
            boot: value.boot,
            awake: value.awake,
            sleep: value.sleep,
            shutdown: value.shutdown,
        }
    }
}

impl From<AuraPowerState> for SlintAuraPowerState {
    fn from(value: AuraPowerState) -> Self {
        let zone = value.zone.into();
        SlintAuraPowerState {
            boot: value.boot,
            awake: value.awake,
            sleep: value.sleep,
            shutdown: value.shutdown,
            zone,
            zone_name_idx: zone as i32,
        }
    }
}

impl From<&AuraPowerState> for SlintAuraPowerState {
    fn from(value: &AuraPowerState) -> Self {
        let zone = value.zone.into();
        SlintAuraPowerState {
            boot: value.boot,
            awake: value.awake,
            sleep: value.sleep,
            shutdown: value.shutdown,
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
        let converted: Vec<SlintAuraPowerState> =
            value.states.iter().map(SlintAuraPowerState::from).collect();
        Self {
            states: ModelRc::from(converted.as_slice()),
        }
    }
}
