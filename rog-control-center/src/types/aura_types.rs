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
use rog_aura::keyboard::{AuraPowerState, LaptopOldAuraPower, LaptopTufAuraPower};
use rog_aura::usb::AuraPowerDev;
use slint::{Model, ModelRc, RgbaColor};

use crate::slint_generatedMainWindow::AuraDevTuf as SlintAuraDevTuf;
impl From<LaptopTufAuraPower> for SlintAuraDevTuf {
    fn from(value: LaptopTufAuraPower) -> Self {
        match value {
            LaptopTufAuraPower::Boot => SlintAuraDevTuf::Boot,
            LaptopTufAuraPower::Awake => SlintAuraDevTuf::Awake,
            LaptopTufAuraPower::Sleep => SlintAuraDevTuf::Sleep,
            LaptopTufAuraPower::Keyboard => SlintAuraDevTuf::Keyboard,
        }
    }
}

impl From<SlintAuraDevTuf> for LaptopTufAuraPower {
    fn from(value: SlintAuraDevTuf) -> Self {
        match value {
            SlintAuraDevTuf::Boot => LaptopTufAuraPower::Boot,
            SlintAuraDevTuf::Awake => LaptopTufAuraPower::Awake,
            SlintAuraDevTuf::Sleep => LaptopTufAuraPower::Sleep,
            SlintAuraDevTuf::Keyboard => LaptopTufAuraPower::Keyboard,
        }
    }
}

use crate::slint_generatedMainWindow::AuraDevRog1 as SlintAuraDevRog1;
impl From<LaptopOldAuraPower> for SlintAuraDevRog1 {
    fn from(value: LaptopOldAuraPower) -> Self {
        match value {
            LaptopOldAuraPower::Awake => SlintAuraDevRog1::Awake,
            LaptopOldAuraPower::Keyboard => SlintAuraDevRog1::Keyboard,
            LaptopOldAuraPower::Lightbar => SlintAuraDevRog1::Lightbar,
            LaptopOldAuraPower::Boot => SlintAuraDevRog1::Boot,
            LaptopOldAuraPower::Sleep => SlintAuraDevRog1::Sleep,
        }
    }
}

impl From<SlintAuraDevRog1> for LaptopOldAuraPower {
    fn from(value: SlintAuraDevRog1) -> Self {
        match value {
            SlintAuraDevRog1::Awake => LaptopOldAuraPower::Awake,
            SlintAuraDevRog1::Keyboard => LaptopOldAuraPower::Keyboard,
            SlintAuraDevRog1::Lightbar => LaptopOldAuraPower::Lightbar,
            SlintAuraDevRog1::Boot => LaptopOldAuraPower::Boot,
            SlintAuraDevRog1::Sleep => LaptopOldAuraPower::Sleep,
        }
    }
}

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
        }
    }
}

use crate::slint_generatedMainWindow::{
    AuraPower as SlintAuraPower, KbAuraPowerState as SlintKbAuraPowerState,
};
impl From<rog_aura::keyboard::LaptopAuraPower> for SlintAuraPower {
    fn from(value: rog_aura::keyboard::LaptopAuraPower) -> Self {
        Self {
            keyboard: SlintKbAuraPowerState {
                awake: value.keyboard.awake,
                boot: value.keyboard.boot,
                shutdown: value.keyboard.shutdown,
                sleep: value.keyboard.sleep,
            },
            lid: SlintKbAuraPowerState {
                awake: value.lid.awake,
                boot: value.lid.boot,
                shutdown: value.lid.shutdown,
                sleep: value.lid.sleep,
            },
            lightbar: SlintKbAuraPowerState {
                awake: value.lightbar.awake,
                boot: value.lightbar.boot,
                shutdown: value.lightbar.shutdown,
                sleep: value.lightbar.sleep,
            },
            logo: SlintKbAuraPowerState {
                awake: value.logo.awake,
                boot: value.logo.boot,
                shutdown: value.logo.shutdown,
                sleep: value.logo.sleep,
            },
            rear_glow: SlintKbAuraPowerState {
                awake: value.rear_glow.awake,
                boot: value.rear_glow.boot,
                shutdown: value.rear_glow.shutdown,
                sleep: value.rear_glow.sleep,
            },
        }
    }
}

impl From<SlintAuraPower> for rog_aura::keyboard::LaptopAuraPower {
    fn from(value: SlintAuraPower) -> Self {
        Self {
            keyboard: AuraPowerState {
                awake: value.keyboard.awake,
                boot: value.keyboard.boot,
                shutdown: value.keyboard.shutdown,
                sleep: value.keyboard.sleep,
            },
            lid: AuraPowerState {
                awake: value.lid.awake,
                boot: value.lid.boot,
                shutdown: value.lid.shutdown,
                sleep: value.lid.sleep,
            },
            lightbar: AuraPowerState {
                awake: value.lightbar.awake,
                boot: value.lightbar.boot,
                shutdown: value.lightbar.shutdown,
                sleep: value.lightbar.sleep,
            },
            logo: AuraPowerState {
                awake: value.logo.awake,
                boot: value.logo.boot,
                shutdown: value.logo.shutdown,
                sleep: value.logo.sleep,
            },
            rear_glow: AuraPowerState {
                awake: value.rear_glow.awake,
                boot: value.rear_glow.boot,
                shutdown: value.rear_glow.shutdown,
                sleep: value.rear_glow.sleep,
            },
        }
    }
}

use crate::slint_generatedMainWindow::AuraPowerDev as SlintAuraPowerDev;
impl From<AuraPowerDev> for SlintAuraPowerDev {
    fn from(value: AuraPowerDev) -> Self {
        let tuf: Vec<SlintAuraDevTuf> = value
            .tuf
            .iter()
            .map(|n| SlintAuraDevTuf::from(*n))
            .collect();
        let old_rog: Vec<SlintAuraDevRog1> = value
            .old_rog
            .iter()
            .map(|n| SlintAuraDevRog1::from(*n))
            .collect();
        Self {
            tuf: ModelRc::from(tuf.as_slice()),
            old_rog: ModelRc::from(old_rog.as_slice()),
            rog: value.rog.into(),
        }
    }
}

impl From<SlintAuraPowerDev> for AuraPowerDev {
    fn from(value: SlintAuraPowerDev) -> Self {
        let tuf: Vec<LaptopTufAuraPower> = value.tuf.iter().map(LaptopTufAuraPower::from).collect();
        let old_rog: Vec<LaptopOldAuraPower> =
            value.old_rog.iter().map(LaptopOldAuraPower::from).collect();
        Self {
            tuf,
            old_rog,
            rog: value.rog.into(),
        }
    }
}
