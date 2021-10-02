#![deny(unused_must_use)]
/// Configuration loading, saving
pub mod config;
/// Control of AniMe matrix display
pub mod ctrl_anime;
/// Keyboard LED brightness control, RGB, and LED display modes
pub mod ctrl_aura;
/// Control of battery charge level
pub mod ctrl_charge;
/// Control CPU min/max freq and turbo, fan mode, fan curves
///
/// Intel machines can control:
/// - CPU min/max frequency
/// - CPU turbo enable/disable
/// - Fan mode (normal, boost, silent)
///
/// AMD machines can control:
/// - CPU turbo enable/disable
/// - Fan mode (normal, boost, silent)
/// - Fan min/max RPM curve
pub mod ctrl_profiles;
/// Control ASUS bios function such as boot sound, Optimus/Dedicated gfx mode
pub mod ctrl_rog_bios;
/// Laptop matching to determine capabilities
pub mod laptops;

/// Fetch all supported functions for the laptop
pub mod ctrl_supported;

pub mod error;

use crate::error::RogError;
use config::Config;
use zbus::ObjectServer;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait Reloadable {
    fn reload(&mut self) -> Result<(), RogError>;
}

pub trait ZbusAdd {
    fn add_to_server(self, server: &mut ObjectServer);
}

pub trait CtrlTask {
    fn do_task(&self) -> Result<(), RogError>;
}

pub trait CtrlTaskComplex {
    type A;

    fn do_task(&mut self, config: &mut Config, event: Self::A);
}

pub trait GetSupported {
    type A;

    fn get_supported() -> Self::A;
}
