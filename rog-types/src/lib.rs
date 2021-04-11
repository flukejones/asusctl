//! This crate is intended for shared types (eg, between daemon and CLI), or
//! for types that might be useful in third-party crates perhaps for
//! sending messages over dbus wire

pub static DBUS_NAME: &str = "org.asuslinux.Daemon";
pub static DBUS_PATH: &str = "/org/asuslinux/Daemon";
pub static DBUS_IFACE: &str = "org.asuslinux.Daemon";

pub mod profile;

pub mod gfx_vendors;

pub mod supported;

pub mod error;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
