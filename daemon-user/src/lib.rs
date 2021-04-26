pub mod user_config;

pub mod error;

pub mod ctrl_anime;

pub mod zbus_anime;

pub static DBUS_NAME: &str = "org.asuslinux.Daemon";

pub static VERSION: &str = env!("CARGO_PKG_VERSION");