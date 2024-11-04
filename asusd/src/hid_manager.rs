// Plan:
// - Manager has udev monitor on USB looking for ROG devices
// - If a device is found, add it to watch
// - Add it to Zbus server
// - If udev sees device removed then remove the zbus path

use std::collections::HashSet;

use log::{debug, error, info, warn};
use mio::{Events, Interest, Poll, Token};
use rog_aura::AuraDeviceType;
use rog_platform::hid_raw::HidRaw;
use rog_platform::keyboard_led::KeyboardBacklight;
use rog_platform::usb_raw::USBRaw;
use tokio::task::spawn_blocking;
use udev::{Device, MonitorBuilder};
use zbus::object_server::SignalContext;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};
use zbus::Connection;

use crate::ctrl_aura::controller::CtrlKbdLed;
use crate::ctrl_aura::trait_impls::{CtrlAuraZbus, AURA_ZBUS_PATH};
use crate::error::RogError;
use crate::{CtrlTask, Reloadable};

// TODO:
// - make this the HID manager (and universal)
// - make aura/anime/slash independent of HID (pass in a HID dev ref)
// - *really* need to make most of this actual kernel drivers
//   - LED class
//   - RGB modes (how, attribute?)
//   - power features (how, attribute?)
//   - what about per-key stuff?
//   - how would the AniMe be exposed? Just a series of LEDs?

pub enum AsusDevice {
    /// The AniMe devices require USBRaw as they are not HID devices
    Usb(USBRaw),
    /// *Most* of the ITE devices will be HIDRaw such as
    /// - Ally gamepad
    /// - Keyboard + LED control
    /// - Some slash devices are also on Keyboard HID
    Hid(HidRaw),
    /// TUF laptops have an aditional set of attributes added to the LED /sysfs/
    TufLedClass(KeyboardBacklight),
    /// TODO
    MulticolourLed,
    None,
}

impl AsusDevice {
    pub fn find_all_hid() -> Result<Vec<Self>, RogError> {
        let mut devices = Vec::new();

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
            if let Ok(dev) = Self::maybe_hid_device(device) {
                devices.push(dev);
            }
        }

        Ok(devices)
    }

    pub fn find_all_devices() -> Result<Vec<Self>, RogError> {
        let mut devices = Vec::new();

        if let Ok(devs) = &mut Self::find_all_hid() {
            devices.append(devs);
        }

        Ok(devices)
    }

    /// For use with udev events
    pub fn maybe_hid_device(device: Device) -> Result<AsusDevice, RogError> {
        // usb_device gives us a product and vendor ID
        if let Some(usb_device) = device.parent_with_subsystem_devtype("usb", "usb_device")? {
            // The asus_wmi driver latches MCU that controls the USB endpoints
            if let Some(parent) = device.parent() {
                if let Some(driver) = parent.driver() {
                    // There is a tree of devices added so filter by driver
                    if driver != "asus" {
                        return Ok(Self::None);
                    }
                } else {
                    return Ok(Self::None);
                }
            }
            // Device is something like 002, while its parent is the MCU
            // Think of it like the device is an endpoint of the USB device attached
            if let Some(usb_id) = usb_device.attribute_value("idProduct") {
                let prod_id = usb_id.to_string_lossy().to_string();
                let aura_device = AuraDeviceType::from(prod_id.as_str());
                if aura_device == AuraDeviceType::Unknown {
                    debug!("Unknown or invalid device: {usb_id:?}, skipping");
                    return Ok(Self::None);
                }
            }

            let dev_node = if let Some(dev_node) = usb_device.devnode() {
                dev_node
            } else {
                debug!("Device has no devnode, skipping");
                return Ok(Self::None);
            };
            info!("AuraControl found device at: {:?}", dev_node);
            return Ok(Self::Hid(HidRaw::from_device(device)?));
        }
        Ok(Self::None)
    }
}

pub struct UdevEventManager {
    _dbus_connection: Connection,
}

impl UdevEventManager {
    pub async fn new(connection: Connection) -> Result<Self, RogError> {
        let conn_copy = connection.clone();
        let mut interfaces = HashSet::new();

        // Do the initial keyboard detection:
        let all = CtrlKbdLed::find_all()?;
        for ctrl in all {
            let path = ctrl.dbus_path.clone();
            interfaces.insert(path.clone()); // ensure we record the initial stuff
            let sig_ctx = CtrlAuraZbus::signal_context(&connection)?;
            let sig_ctx2 = sig_ctx.clone();
            let zbus = CtrlAuraZbus::new(ctrl, sig_ctx);
            start_tasks(zbus, connection.clone(), sig_ctx2, path).await?;
        }

        let manager = Self {
            _dbus_connection: connection,
        };

        // TODO: The /sysfs/ LEDs don't cause events, so they need to be manually
        // checked for and added

        // detect all plugged in aura devices (eventually)
        // only USB devices are detected for here
        spawn_blocking(move || {
            let mut monitor = MonitorBuilder::new()?.match_subsystem("hidraw")?.listen()?;
            let mut poll = Poll::new()?;
            let mut events = Events::with_capacity(1024);
            poll.registry()
                .register(&mut monitor, Token(0), Interest::READABLE)?;

            loop {
                if poll.poll(&mut events, None).is_err() {
                    continue;
                }
                for event in monitor.iter() {
                    let action = event.action().unwrap_or_default();

                    if let Some(parent) =
                        event.parent_with_subsystem_devtype("usb", "usb_device")?
                    {
                        if action == "remove" {
                            if let Some(path) = dbus_path_for_dev(&parent) {
                                if interfaces.remove(&path) {
                                    info!("AuraManager removing: {path:?}");
                                    let conn_copy = conn_copy.clone();
                                    tokio::spawn(async move {
                                        let res = conn_copy
                                            .object_server()
                                            .remove::<CtrlAuraZbus, _>(&path)
                                            .await
                                            .map_err(|e| {
                                                error!("Failed to remove {path:?}, {e:?}");
                                                e
                                            })?;
                                        info!("AuraManager removed: {path:?}, {res}");
                                        Ok::<(), RogError>(())
                                    });
                                }
                            }
                        } else if action == "add" {
                            // TODO:
                            // - check the VID
                            // - ask subsystems if they can use it
                            if let Ok(Some(ctrl)) =
                                CtrlKbdLed::maybe_device(event.device(), &mut interfaces)
                            {
                                ctrl.add_to_dbus_and_start(&mut interfaces, conn_copy.clone())
                                    .map_err(|e| {
                                        error!("Couldn't start aura device on dbus: {e:?}")
                                    })
                                    .ok();
                            }
                        };
                    }
                }
            }
            // Required for return type on spawn
            #[allow(unreachable_code)]
            Ok::<(), RogError>(())
        });
        Ok(manager)
    }
}

/// Returns only the Device details concatenated in a form usable for
/// adding/appending to a filename
fn filename_partial(parent: &Device) -> Option<OwnedObjectPath> {
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

pub(crate) fn dbus_path_for_dev(parent: &Device) -> Option<OwnedObjectPath> {
    if let Some(filename) = filename_partial(parent) {
        return Some(
            ObjectPath::from_str_unchecked(&format!("{AURA_ZBUS_PATH}/{filename}")).into(),
        );
    }
    None
}

pub(crate) fn dbus_path_for_tuf() -> OwnedObjectPath {
    ObjectPath::from_str_unchecked(&format!("{AURA_ZBUS_PATH}/tuf")).into()
}

pub async fn start_tasks(
    mut zbus: CtrlAuraZbus,
    connection: Connection,
    _signal_ctx: SignalContext<'static>,
    path: OwnedObjectPath,
) -> Result<(), RogError> {
    // let task = zbus.clone();
    // let signal_ctx = signal_ctx.clone();
    zbus.reload()
        .await
        .unwrap_or_else(|err| warn!("Controller error: {}", err));
    connection
        .object_server()
        .at(path.clone(), zbus)
        .await
        .map_err(|e| error!("Couldn't add server at path: {path}, {e:?}"))
        .ok();
    // TODO: skip this until we keep handles to tasks so they can be killed
    // task.create_tasks(signal_ctx).await
    Ok(())
}
