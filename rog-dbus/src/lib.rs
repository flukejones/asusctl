pub use asusd::{DBUS_IFACE, DBUS_NAME, DBUS_PATH};

pub mod asus_armoury;
pub mod scsi_aura;
pub mod zbus_anime;
pub mod zbus_aura;
pub mod zbus_fan_curves;
pub mod zbus_platform;
pub mod zbus_slash;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn list_iface_blocking() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::system()?;
    let f = zbus::blocking::fdo::ObjectManagerProxy::new(&conn, "xyz.ljones.Asusd", "/")?;
    let interfaces = f.get_managed_objects()?;
    let mut ifaces = Vec::new();
    for v in interfaces.iter() {
        for k in v.1.keys() {
            ifaces.push(k.to_string());
        }
    }
    Ok(ifaces)
}

pub fn has_iface_blocking(iface: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::system()?;
    let f = zbus::blocking::fdo::ObjectManagerProxy::new(&conn, "xyz.ljones.Asusd", "/")?;
    let interfaces = f.get_managed_objects()?;
    for v in interfaces.iter() {
        for k in v.1.keys() {
            if k.as_str() == iface {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub async fn has_iface(iface: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let conn = zbus::Connection::system().await?;
    let f = zbus::fdo::ObjectManagerProxy::new(&conn, "xyz.ljones.Asusd", "/").await?;
    let interfaces = f.get_managed_objects().await?;
    for v in interfaces.iter() {
        for k in v.1.keys() {
            if k.as_str() == iface {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
