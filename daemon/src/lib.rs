#![deny(unused_must_use)]
/// Configuration loading, saving
pub mod config;
///
pub mod ctrl_anime;
///
pub mod ctrl_charge;
///
pub mod ctrl_fan_cpu;
///
pub mod ctrl_gfx;
///
pub mod ctrl_leds;
/// Control ASUS bios function such as boot sound, Optimus/Dedicated gfx mode
pub mod ctrl_rog_bios;
/// Laptop matching to determine capabilities
pub mod laptops;

/// Fetch all supported functions for the laptop
pub mod supported;

mod error;

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
    fn do_task(&mut self) -> Result<(), RogError>;
}

pub trait CtrlTaskComplex {
    type A;

    fn do_task(&mut self, config: &mut Config, event: Self::A);
}

pub trait GetSupported {
    type A;

    fn get_supported() -> Self::A;
}
