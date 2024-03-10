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

use rog_aura::power::KbAuraPowerState;
use slint::{Model, ModelRc, RgbaColor};

use crate::slint_generatedMainWindow::AuraDevTuf as SlintAuraDevTuf;
impl From<rog_aura::usb::AuraDevTuf> for SlintAuraDevTuf {
    fn from(value: rog_aura::usb::AuraDevTuf) -> Self {
        match value {
            rog_aura::usb::AuraDevTuf::Boot => SlintAuraDevTuf::Boot,
            rog_aura::usb::AuraDevTuf::Awake => SlintAuraDevTuf::Awake,
            rog_aura::usb::AuraDevTuf::Sleep => SlintAuraDevTuf::Sleep,
            rog_aura::usb::AuraDevTuf::Keyboard => SlintAuraDevTuf::Keyboard,
        }
    }
}

impl From<SlintAuraDevTuf> for rog_aura::usb::AuraDevTuf {
    fn from(value: SlintAuraDevTuf) -> Self {
        match value {
            SlintAuraDevTuf::Boot => rog_aura::usb::AuraDevTuf::Boot,
            SlintAuraDevTuf::Awake => rog_aura::usb::AuraDevTuf::Awake,
            SlintAuraDevTuf::Sleep => rog_aura::usb::AuraDevTuf::Sleep,
            SlintAuraDevTuf::Keyboard => rog_aura::usb::AuraDevTuf::Keyboard,
        }
    }
}

use crate::slint_generatedMainWindow::AuraDevRog1 as SlintAuraDevRog1;
impl From<rog_aura::usb::AuraDevRog1> for SlintAuraDevRog1 {
    fn from(value: rog_aura::usb::AuraDevRog1) -> Self {
        match value {
            rog_aura::usb::AuraDevRog1::Awake => SlintAuraDevRog1::Awake,
            rog_aura::usb::AuraDevRog1::Keyboard => SlintAuraDevRog1::Keyboard,
            rog_aura::usb::AuraDevRog1::Lightbar => SlintAuraDevRog1::Lightbar,
            rog_aura::usb::AuraDevRog1::Boot => SlintAuraDevRog1::Boot,
            rog_aura::usb::AuraDevRog1::Sleep => SlintAuraDevRog1::Sleep,
        }
    }
}

impl From<SlintAuraDevRog1> for rog_aura::usb::AuraDevRog1 {
    fn from(value: SlintAuraDevRog1) -> Self {
        match value {
            SlintAuraDevRog1::Awake => rog_aura::usb::AuraDevRog1::Awake,
            SlintAuraDevRog1::Keyboard => rog_aura::usb::AuraDevRog1::Keyboard,
            SlintAuraDevRog1::Lightbar => rog_aura::usb::AuraDevRog1::Lightbar,
            SlintAuraDevRog1::Boot => rog_aura::usb::AuraDevRog1::Boot,
            SlintAuraDevRog1::Sleep => rog_aura::usb::AuraDevRog1::Sleep,
        }
    }
}

use crate::slint_generatedMainWindow::PowerZones as SlintPowerZones;
impl From<rog_aura::aura_detection::PowerZones> for SlintPowerZones {
    fn from(value: rog_aura::aura_detection::PowerZones) -> Self {
        match value {
            rog_aura::aura_detection::PowerZones::Logo => SlintPowerZones::Logo,
            rog_aura::aura_detection::PowerZones::Keyboard => SlintPowerZones::Keyboard,
            rog_aura::aura_detection::PowerZones::Lightbar => SlintPowerZones::Lightbar,
            rog_aura::aura_detection::PowerZones::Lid => SlintPowerZones::Lid,
            rog_aura::aura_detection::PowerZones::RearGlow => SlintPowerZones::RearGlow,
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
impl From<rog_aura::power::AuraPower> for SlintAuraPower {
    fn from(value: rog_aura::power::AuraPower) -> Self {
        Self {
            keyboard: SlintKbAuraPowerState {
                awake: value.keyboard.awake,
                boot: value.keyboard.boot,
                shutdown: value.keyboard.shutdown,
                sleep: value.keyboard.sleep,
                zone: value.keyboard.zone.into(),
            },
            lid: SlintKbAuraPowerState {
                awake: value.lid.awake,
                boot: value.lid.boot,
                shutdown: value.lid.shutdown,
                sleep: value.lid.sleep,
                zone: value.lid.zone.into(),
            },
            lightbar: SlintKbAuraPowerState {
                awake: value.lightbar.awake,
                boot: value.lightbar.boot,
                shutdown: value.lightbar.shutdown,
                sleep: value.lightbar.sleep,
                zone: value.lightbar.zone.into(),
            },
            logo: SlintKbAuraPowerState {
                awake: value.logo.awake,
                boot: value.logo.boot,
                shutdown: value.logo.shutdown,
                sleep: value.logo.sleep,
                zone: value.logo.zone.into(),
            },
            rear_glow: SlintKbAuraPowerState {
                awake: value.rear_glow.awake,
                boot: value.rear_glow.boot,
                shutdown: value.rear_glow.shutdown,
                sleep: value.rear_glow.sleep,
                zone: value.rear_glow.zone.into(),
            },
        }
    }
}

impl From<SlintAuraPower> for rog_aura::power::AuraPower {
    fn from(value: SlintAuraPower) -> Self {
        Self {
            keyboard: KbAuraPowerState {
                awake: value.keyboard.awake,
                boot: value.keyboard.boot,
                shutdown: value.keyboard.shutdown,
                sleep: value.keyboard.sleep,
                zone: value.keyboard.zone.into(),
            },
            lid: KbAuraPowerState {
                awake: value.lid.awake,
                boot: value.lid.boot,
                shutdown: value.lid.shutdown,
                sleep: value.lid.sleep,
                zone: value.lid.zone.into(),
            },
            lightbar: KbAuraPowerState {
                awake: value.lightbar.awake,
                boot: value.lightbar.boot,
                shutdown: value.lightbar.shutdown,
                sleep: value.lightbar.sleep,
                zone: value.lightbar.zone.into(),
            },
            logo: KbAuraPowerState {
                awake: value.logo.awake,
                boot: value.logo.boot,
                shutdown: value.logo.shutdown,
                sleep: value.logo.sleep,
                zone: value.logo.zone.into(),
            },
            rear_glow: KbAuraPowerState {
                awake: value.rear_glow.awake,
                boot: value.rear_glow.boot,
                shutdown: value.rear_glow.shutdown,
                sleep: value.rear_glow.sleep,
                zone: value.rear_glow.zone.into(),
            },
        }
    }
}

use crate::slint_generatedMainWindow::AuraPowerDev as SlintAuraPowerDev;
impl From<rog_aura::usb::AuraPowerDev> for SlintAuraPowerDev {
    fn from(value: rog_aura::usb::AuraPowerDev) -> Self {
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

impl From<SlintAuraPowerDev> for rog_aura::usb::AuraPowerDev {
    fn from(value: SlintAuraPowerDev) -> Self {
        let tuf: Vec<rog_aura::usb::AuraDevTuf> = value
            .tuf
            .iter()
            .map(rog_aura::usb::AuraDevTuf::from)
            .collect();
        let old_rog: Vec<rog_aura::usb::AuraDevRog1> = value
            .old_rog
            .iter()
            .map(rog_aura::usb::AuraDevRog1::from)
            .collect();
        Self {
            tuf,
            old_rog,
            rog: value.rog.into(),
        }
    }
}
