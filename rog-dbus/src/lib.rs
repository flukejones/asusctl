pub use asusd::{DBUS_IFACE, DBUS_NAME, DBUS_PATH};

pub mod zbus_anime;
pub mod zbus_aura;
pub mod zbus_fan_curves;
pub mod zbus_platform;
pub mod zbus_slash;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
