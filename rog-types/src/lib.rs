//! This crate is intended for shared types (eg, between daemon and CLI), or
//! for types that might be useful in third-party crates perhaps for
//! sending messages over dbus wire

pub static DBUS_NAME: &str = "org.asuslinux.Daemon";
pub static DBUS_PATH: &str = "/org/asuslinux/Daemon";
pub static DBUS_IFACE: &str = "org.asuslinux.Daemon";
pub const LED_MSG_LEN: usize = 17;

pub mod aura_modes;

pub mod profile;

/// Contains mostly only what is required for parsing CLI options
pub mod cli_options;

/// Enables you to create fancy RGB effects
pub mod aura_perkey;

/// Helper functions for the AniMe display
pub mod anime_matrix;

pub mod gfx_vendors;

pub mod error;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
