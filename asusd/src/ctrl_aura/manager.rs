// Plan:
// - Manager has udev monitor on USB looking for ROG devices
// - If a device is found, add it to watch
// - Add it to Zbus server
// - If udev sees device removed then remove the zbus path

use std::collections::HashSet;

use log::{debug, error, info, warn};
use mio::{Events, Interest, Poll, Token};
use rog_aura::aura_detection::LaptopLedData;
use rog_aura::usb::AuraDevice;
use rog_platform::hid_raw::HidRaw;
use tokio::task::spawn_blocking;
use udev::{Device, MonitorBuilder};
// use zbus::fdo::ObjectManager;
use zbus::object_server::SignalContext;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};
use zbus::Connection;

use crate::ctrl_aura::controller::CtrlKbdLed;
use crate::ctrl_aura::trait_impls::{CtrlAuraZbus, AURA_ZBUS_PATH};
use crate::error::RogError;
use crate::{CtrlTask, Reloadable};

pub struct AuraManager {
    _connection: Connection,
}

impl AuraManager {
    pub async fn new(connection: Connection) -> Result<Self, RogError> {
        let conn_copy = connection.clone();
        let data = LaptopLedData::get_data();
        let mut interfaces = HashSet::new();

        // Do the initial keyboard detection:
        let all = CtrlKbdLed::find_all(&data)?;
        for ctrl in all {
            let path = ctrl.dbus_path.clone();
            interfaces.insert(path.clone()); // ensure we record the initial stuff
            let sig_ctx = CtrlAuraZbus::signal_context(&connection)?;
            let sig_ctx2 = sig_ctx.clone();
            let zbus = CtrlAuraZbus::new(ctrl, sig_ctx);
            start_tasks(zbus, connection.clone(), sig_ctx2, path).await?;
        }

        let manager = Self {
            _connection: connection,
        };

        // detect all plugged in aura devices (eventually)
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
                    let parent = if let Some(parent) =
                        event.parent_with_subsystem_devtype("usb", "usb_device")?
                    {
                        parent
                    } else {
                        continue;
                    };

                    let action = if let Some(action) = event.action() {
                        action
                    } else {
                        continue;
                    };

                    let id_product = if let Some(id_product) = parent.attribute_value("idProduct") {
                        id_product.to_string_lossy()
                    } else {
                        continue;
                    };

                    let path = if let Some(path) = dbus_path_for_dev(&parent) {
                        path
                    } else {
                        continue;
                    };

                    let aura_device = AuraDevice::from(&*id_product);
                    if aura_device == AuraDevice::Unknown {
                        warn!("idProduct:{id_product:?} is unknown, not using");
                        continue;
                    }

                    if action == "remove" {
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
                    } else if action == "add" {
                        if interfaces.contains(&path) {
                            debug!("Already a ctrl at {path:?}");
                            continue;
                        }

                        // Need to check the driver is asus to prevent using hid_generic
                        if let Some(p2) = event.parent() {
                            if let Some(driver) = p2.driver() {
                                // There is a tree of devices added so filter by driver
                                if driver != "asus" {
                                    debug!("{id_product:?} driver was not asus, skipping");
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }

                        if let Some(dev_node) = event.devnode() {
                            if let Ok(raw) = HidRaw::from_device(event.device())
                                .map_err(|e| error!("device path error: {e:?}"))
                            {
                                if let Ok(mut ctrl) =
                                    CtrlKbdLed::from_hidraw(raw, path.clone(), &data)
                                {
                                    ctrl.config = CtrlKbdLed::init_config(aura_device, &data);
                                    interfaces.insert(path.clone());
                                    info!("AuraManager starting device at: {dev_node:?}, {path:?}");
                                    let sig_ctx = CtrlAuraZbus::signal_context(&conn_copy)?;
                                    let zbus = CtrlAuraZbus::new(ctrl, sig_ctx);
                                    let sig_ctx = CtrlAuraZbus::signal_context(&conn_copy)?;
                                    let conn_copy = conn_copy.clone();
                                    tokio::spawn(async move {
                                        start_tasks(zbus, conn_copy.clone(), sig_ctx, path).await
                                    });
                                }
                            }
                        }
                    };
                }
            }
            // Required for return type on spawn
            #[allow(unreachable_code)]
            Ok::<(), RogError>(())
        });
        Ok(manager)
    }
}

pub(crate) fn dbus_path_for_dev(parent: &Device) -> Option<OwnedObjectPath> {
    if let Some(id_product) = parent.attribute_value("idProduct") {
        let id_product = id_product.to_string_lossy();
        let path = if let Some(devnum) = parent.attribute_value("devnum") {
            let devnum = devnum.to_string_lossy();
            if let Some(devpath) = parent.attribute_value("devpath") {
                let devpath = devpath.to_string_lossy();
                format!("{AURA_ZBUS_PATH}/{id_product}_{devnum}_{devpath}")
            } else {
                format!("{AURA_ZBUS_PATH}/{id_product}_{devnum}")
            }
        } else {
            format!("{AURA_ZBUS_PATH}/{id_product}")
        };
        return Some(ObjectPath::from_str_unchecked(&path).into());
    }
    None
}

async fn start_tasks(
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
    connection.object_server().at(path, zbus).await.unwrap();
    // TODO: skip this until we keep handles to tasks so they can be killed
    // task.create_tasks(signal_ctx).await
    Ok(())
}
