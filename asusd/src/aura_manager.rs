// Plan:
// - Manager has udev monitor on USB looking for ROG devices
// - If a device is found, add it to watch
// - Add it to Zbus server
// - If udev sees device removed then remove the zbus path

use std::sync::Arc;

use dmi_id::DMIID;
use futures_lite::future::block_on;
use log::{debug, error, info, warn};
use mio::{Events, Interest, Poll, Token};
use rog_platform::error::PlatformError;
use rog_platform::hid_raw::HidRaw;
use tokio::sync::Mutex;
use udev::{Device, MonitorBuilder};
use zbus::zvariant::{ObjectPath, OwnedObjectPath};
use zbus::Connection;

use crate::aura_anime::trait_impls::AniMeZbus;
use crate::aura_laptop::trait_impls::AuraZbus;
use crate::aura_scsi::trait_impls::ScsiZbus;
use crate::aura_slash::trait_impls::SlashZbus;
use crate::aura_types::DeviceHandle;
use crate::error::RogError;
use crate::ASUS_ZBUS_PATH;

const MOD_NAME: &str = "aura";

/// Returns only the Device details concatenated in a form usable for
/// adding/appending to a filename
pub fn filename_partial(parent: &Device) -> Option<OwnedObjectPath> {
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
            path.replace('.', "").clone_into(&mut path);
        }
        return Some(ObjectPath::from_str_unchecked(&path).into());
    }
    None
}

fn dbus_path_for_dev(parent: &Device) -> Option<OwnedObjectPath> {
    if let Some(filename) = filename_partial(parent) {
        return Some(
            ObjectPath::from_str_unchecked(&format!("{ASUS_ZBUS_PATH}/{MOD_NAME}/{filename}"))
                .into()
        );
    }
    None
}

fn dbus_path_for_tuf() -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!("{ASUS_ZBUS_PATH}/{MOD_NAME}/tuf")).into()
}

fn dbus_path_for_slash() -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!("{ASUS_ZBUS_PATH}/{MOD_NAME}/slash")).into()
}

fn dbus_path_for_anime() -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!("{ASUS_ZBUS_PATH}/{MOD_NAME}/anime")).into()
}

fn dbus_path_for_scsi(prod_id: &str) -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!("{ASUS_ZBUS_PATH}/{MOD_NAME}/{prod_id}_scsi")).into()
}

fn dev_prop_matches(dev: &Device, prop: &str, value: &str) -> bool {
    if let Some(p) = dev.property_value(prop) {
        return p == value;
    }
    false
}

/// A device.
///
/// Each controller within should track its dbus path so it can be removed if
/// required.
pub struct AsusDevice {
    device: DeviceHandle,
    dbus_path: OwnedObjectPath
}

pub struct DeviceManager {
    _dbus_connection: Connection
}

impl DeviceManager {
    async fn init_hid_devices(
        connection: &Connection,
        device: Device
    ) -> Result<Vec<AsusDevice>, RogError> {
        let mut devices = Vec::new();
        if let Some(usb_device) = device.parent_with_subsystem_devtype("usb", "usb_device")? {
            if let Some(usb_id) = usb_device.attribute_value("idProduct") {
                if let Some(vendor_id) = usb_device.attribute_value("idVendor") {
                    if vendor_id != "0b05" {
                        debug!("Not ASUS vendor ID");
                        return Ok(devices);
                    }
                    // Almost all devices are identified by the productId.
                    // So let's see what we have and:
                    // 1. Generate an interface path
                    // 2. Create the device
                    // Use the top-level endpoint, not the parent
                    if let Ok(hidraw) = HidRaw::from_device(device) {
                        debug!("Testing device {usb_id:?}");
                        let dev = Arc::new(Mutex::new(hidraw));
                        // SLASH DEVICE
                        if let Ok(dev_type) = DeviceHandle::new_slash_hid(
                            dev.clone(),
                            usb_id.to_str().unwrap_or_default()
                        )
                        .await
                        {
                            if let DeviceHandle::Slash(slash) = dev_type.clone() {
                                let path =
                                    dbus_path_for_dev(&usb_device).unwrap_or(dbus_path_for_slash());
                                let ctrl = SlashZbus::new(slash);
                                ctrl.start_tasks(connection, path.clone()).await.unwrap();
                                devices.push(AsusDevice {
                                    device: dev_type,
                                    dbus_path: path
                                });
                            }
                        }
                        // ANIME MATRIX DEVICE
                        if let Ok(dev_type) = DeviceHandle::maybe_anime_hid(
                            dev.clone(),
                            usb_id.to_str().unwrap_or_default()
                        )
                        .await
                        {
                            if let DeviceHandle::AniMe(anime) = dev_type.clone() {
                                let path =
                                    dbus_path_for_dev(&usb_device).unwrap_or(dbus_path_for_anime());
                                let ctrl = AniMeZbus::new(anime);
                                ctrl.start_tasks(connection, path.clone()).await.unwrap();
                                devices.push(AsusDevice {
                                    device: dev_type,
                                    dbus_path: path
                                });
                            }
                        }
                        // AURA LAPTOP DEVICE
                        if let Ok(dev_type) = DeviceHandle::maybe_laptop_aura(
                            Some(dev),
                            usb_id.to_str().unwrap_or_default()
                        )
                        .await
                        {
                            if let DeviceHandle::Aura(aura) = dev_type.clone() {
                                let path =
                                    dbus_path_for_dev(&usb_device).unwrap_or(dbus_path_for_tuf());
                                let ctrl = AuraZbus::new(aura);
                                ctrl.start_tasks(connection, path.clone()).await.unwrap();
                                devices.push(AsusDevice {
                                    device: dev_type,
                                    dbus_path: path
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(devices)
    }

    /// To be called on daemon startup
    async fn init_all_hid(connection: &Connection) -> Result<Vec<AsusDevice>, RogError> {
        // track and ensure we use only one hidraw per prod_id
        // let mut interfaces = HashSet::new();
        let mut devices: Vec<AsusDevice> = Vec::new();

        let mut enumerator = udev::Enumerator::new().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("enumerator failed".into(), err)
        })?;

        enumerator.match_subsystem("hidraw").map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("match_subsystem failed".into(), err)
        })?;

        for device in enumerator
            .scan_devices()
            .map_err(|e| PlatformError::IoPath("enumerator".to_owned(), e))?
        {
            devices.append(&mut Self::init_hid_devices(connection, device).await?);
        }

        Ok(devices)
    }

    async fn init_scsi(
        connection: &Connection,
        device: &Device,
        path: OwnedObjectPath
    ) -> Option<AsusDevice> {
        // "ID_MODEL_ID" "1932"
        // "ID_VENDOR_ID" "0b05"
        if dev_prop_matches(device, "ID_VENDOR_ID", "0b05") {
            if let Some(dev_node) = device.devnode() {
                let prod_id = device
                    .property_value("ID_MODEL_ID")
                    .unwrap_or_default()
                    .to_string_lossy();
                if let Ok(dev_type) =
                    DeviceHandle::maybe_scsi(dev_node.as_os_str().to_str().unwrap(), &prod_id).await
                {
                    if let DeviceHandle::Scsi(scsi) = dev_type.clone() {
                        let ctrl = ScsiZbus::new(scsi);
                        ctrl.start_tasks(connection, path.clone()).await.unwrap();
                        return Some(AsusDevice {
                            device: dev_type,
                            dbus_path: path
                        });
                    }
                }
            }
        }
        None
    }

    async fn init_all_scsi(connection: &Connection) -> Result<Vec<AsusDevice>, RogError> {
        // track and ensure we use only one hidraw per prod_id
        // let mut interfaces = HashSet::new();
        let mut devices: Vec<AsusDevice> = Vec::new();

        let mut enumerator = udev::Enumerator::new().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("enumerator failed".into(), err)
        })?;

        enumerator.match_subsystem("block").map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("match_subsystem failed".into(), err)
        })?;

        let mut found = Vec::new();
        for device in enumerator
            .scan_devices()
            .map_err(|e| PlatformError::IoPath("enumerator".to_owned(), e))?
        {
            if let Some(serial) = device.property_value("ID_SERIAL_SHORT") {
                let serial = serial.to_string_lossy().to_string();
                let path = dbus_path_for_scsi(&serial);
                if found.contains(&path) {
                    continue;
                }

                if let Some(dev) = Self::init_scsi(connection, &device, path.clone()).await {
                    devices.push(dev);
                    found.push(path);
                }
            } else {
                debug!("No serial for SCSI device: {:?}", device.devpath());
            }
        }

        Ok(devices)
    }

    pub async fn find_all_devices(connection: &Connection) -> Vec<AsusDevice> {
        let mut devices: Vec<AsusDevice> = Vec::new();
        // HID first, always
        if let Ok(devs) = &mut Self::init_all_hid(connection).await {
            devices.append(devs);
        }
        // USB after, need to check if HID picked something up and if so, skip it
        let mut do_anime = true;
        let mut do_slash = true;
        let mut do_kb_backlight = true;
        for dev in devices.iter() {
            if matches!(dev.device, DeviceHandle::Slash(_)) {
                do_slash = false;
            }
            if matches!(dev.device, DeviceHandle::AniMe(_)) {
                do_anime = false;
            }
            if matches!(dev.device, DeviceHandle::Aura(_) | DeviceHandle::OldAura(_)) {
                do_kb_backlight = false;
            }
        }

        if do_slash {
            if let Ok(dev_type) = DeviceHandle::new_slash_usb().await {
                if let DeviceHandle::Slash(slash) = dev_type.clone() {
                    let path = dbus_path_for_slash();
                    let ctrl = SlashZbus::new(slash);
                    ctrl.start_tasks(connection, path.clone()).await.unwrap();
                    devices.push(AsusDevice {
                        device: dev_type,
                        dbus_path: path
                    });
                }
            } else {
                info!("Tested device was not Slash");
            }
        }

        if do_anime {
            if let Ok(dev_type) = DeviceHandle::maybe_anime_usb().await {
                // TODO: this is copy/pasted
                if let DeviceHandle::AniMe(anime) = dev_type.clone() {
                    let path = dbus_path_for_anime();
                    let ctrl = AniMeZbus::new(anime);
                    if ctrl
                        .start_tasks(connection, path.clone())
                        .await
                        .map_err(|e| error!("Failed to start tasks: {e:?}, not adding this device"))
                        .is_ok()
                    {
                        devices.push(AsusDevice {
                            device: dev_type,
                            dbus_path: path
                        });
                    }
                }
            } else {
                info!("Tested device was not AniMe Matrix");
            }
        }

        if do_kb_backlight {
            // TUF AURA LAPTOP DEVICE
            // product_name = ASUS TUF Gaming F15 FX507ZE_FX507ZE
            // product_family = ASUS TUF Gaming F15
            let product_name = DMIID::new().unwrap_or_default().product_name;
            let product_family = DMIID::new().unwrap_or_default().product_family;
            info!(
                "No USB keyboard aura, system is {product_name}, try using sysfs backlight control"
            );
            if product_name.contains("TUF") || product_family.contains("TUF") {
                info!("TUF laptop, try using sysfs backlight control");
                if let Ok(dev_type) = DeviceHandle::maybe_laptop_aura(None, "tuf").await {
                    if let DeviceHandle::Aura(aura) = dev_type.clone() {
                        let path = dbus_path_for_tuf();
                        let ctrl = AuraZbus::new(aura);
                        ctrl.start_tasks(connection, path.clone()).await.unwrap();
                        devices.push(AsusDevice {
                            device: dev_type,
                            dbus_path: path
                        });
                    }
                }
            }
        }

        if let Ok(devs) = &mut Self::init_all_scsi(connection).await {
            devices.append(devs);
        }

        devices
    }

    pub async fn new(connection: Connection) -> Result<Self, RogError> {
        let conn_copy = connection.clone();
        let devices = Self::find_all_devices(&conn_copy).await;
        info!("Found {} valid devices on startup", devices.len());
        let devices = Arc::new(Mutex::new(devices));
        let manager = Self {
            _dbus_connection: connection
        };

        // TODO: The /sysfs/ LEDs don't cause events, so they need to be manually
        // checked for and added

        std::thread::spawn(move || {
            let mut monitor = MonitorBuilder::new()?.listen()?;
            let mut poll = Poll::new()?;
            let mut events = Events::with_capacity(1024);
            poll.registry()
                .register(&mut monitor, Token(0), Interest::READABLE)?;

            let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
            let _enter = rt.enter();
            loop {
                if poll.poll(&mut events, None).is_err() {
                    continue;
                }
                for event in monitor.iter() {
                    let action = event
                        .action()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let subsys = if let Some(subsys) = event.subsystem() {
                        subsys.to_string_lossy().to_string()
                    } else {
                        continue;
                    };

                    let devices = devices.clone();
                    let conn_copy = conn_copy.clone();
                    block_on(async move {
                        // SCSCI devs
                        if subsys == "block" {
                            if action == "remove" {
                                if let Some(serial) =
                                    event.device().property_value("ID_SERIAL_SHORT")
                                {
                                    let serial = serial.to_string_lossy().to_string();
                                    let path = dbus_path_for_scsi(&serial);

                                    let index = if let Some(index) = devices
                                        .lock()
                                        .await
                                        .iter()
                                        .position(|dev| dev.dbus_path == path)
                                    {
                                        index
                                    } else {
                                        if dev_prop_matches(&event.device(), "ID_VENDOR_ID", "0b05")
                                        {
                                            warn!("No device for dbus path: {path:?}");
                                        }
                                        return Ok(());
                                    };
                                    info!("removing: {path:?}");
                                    let dev = devices.lock().await.remove(index);
                                    let path = path.clone();
                                    if let DeviceHandle::Scsi(_) = dev.device {
                                        conn_copy
                                            .object_server()
                                            .remove::<ScsiZbus, _>(&path)
                                            .await?;
                                    }
                                }
                            } else if action == "add" {
                                let evdev = event.device();
                                if let Some(serial) = evdev.property_value("ID_SERIAL_SHORT") {
                                    let serial = serial.to_string_lossy().to_string();
                                    let path = dbus_path_for_scsi(&serial);
                                    if let Some(new_devs) =
                                        Self::init_scsi(&conn_copy, &evdev, path).await
                                    {
                                        devices.lock().await.append(&mut vec![new_devs]);
                                    }
                                }
                            };
                        }

                        if subsys == "hidraw" {
                            if let Some(parent) =
                                event.parent_with_subsystem_devtype("usb", "usb_device")?
                            {
                                if action == "remove" {
                                    if let Some(path) = dbus_path_for_dev(&parent) {
                                        // Find the indexs of devices matching the path
                                        let removals: Vec<usize> = devices
                                            .lock()
                                            .await
                                            .iter()
                                            .enumerate()
                                            .filter_map(|(i, dev)| {
                                                if dev.dbus_path == path {
                                                    Some(i)
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();
                                        if removals.is_empty() {
                                            return Ok(());
                                        }
                                        info!("removing: {path:?}");
                                        // Iter in reverse so as to not screw up indexing
                                        for index in removals.iter().rev() {
                                            let dev = devices.lock().await.remove(*index);
                                            let path = path.clone();
                                            let res = match dev.device {
                                                DeviceHandle::Aura(_) => {
                                                    conn_copy
                                                        .object_server()
                                                        .remove::<AuraZbus, _>(&path)
                                                        .await?
                                                }
                                                DeviceHandle::Slash(_) => {
                                                    conn_copy
                                                        .object_server()
                                                        .remove::<SlashZbus, _>(&path)
                                                        .await?
                                                }
                                                DeviceHandle::AniMe(_) => {
                                                    conn_copy
                                                        .object_server()
                                                        .remove::<AniMeZbus, _>(&path)
                                                        .await?
                                                }
                                                DeviceHandle::Scsi(_) => {
                                                    conn_copy
                                                        .object_server()
                                                        .remove::<ScsiZbus, _>(&path)
                                                        .await?
                                                }
                                                _ => todo!()
                                            };
                                            info!("AuraManager removed: {path:?}, {res}");
                                        }
                                    }
                                } else if action == "add" {
                                    let evdev = event.device();
                                    if let Ok(mut new_devs) =
                                        Self::init_hid_devices(&conn_copy, evdev)
                                            .await
                                            .map_err(|e| error!("Couldn't add new device: {e:?}"))
                                    {
                                        devices.lock().await.append(&mut new_devs);
                                    }
                                };
                            }
                        }
                        Ok::<(), RogError>(())
                    })
                    .map_err(|e| error!("{e:?}"))
                    .ok();
                }
            }
            // Required for return type on spawn
            #[allow(unreachable_code)]
            Ok::<(), RogError>(())
        });
        Ok(manager)
    }
}
