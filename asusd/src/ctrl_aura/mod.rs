use log::warn;
use udev::Device;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

pub mod config;
pub mod controller;
pub mod manager;
/// Implements `CtrlTask`, `Reloadable`, `ZbusRun`
pub mod trait_impls;

/// Returns only the Device details concatenated in a form usable for
/// adding/appending to a filename
pub(super) fn filename_partial(parent: &Device) -> Option<OwnedObjectPath> {
    if let Some(id_product) = parent.attribute_value("idProduct") {
        let id_product = id_product.to_string_lossy();
        let mut path = if let Some(devnum) = parent.attribute_value("devnum") {
            let devnum = devnum.to_string_lossy();
            if let Some(devpath) = parent.attribute_value("devpath") {
                let devpath = devpath.to_string_lossy();
                format!("{id_product}_{devnum}_{devpath}")
            } else {
                format!("{id_product}_{devnum}")
            }
        } else {
            format!("{id_product}")
        };
        if path.contains('.') {
            warn!("dbus path for {id_product} contains `.`, removing");
            path = path.replace('.', "").to_owned();
        }
        return Some(ObjectPath::from_str_unchecked(&path).into());
    }
    None
}
