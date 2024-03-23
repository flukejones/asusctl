// Plan:
// - Manager has udev monitor on USB looking for ROG devices
// - If a device is found, add it to watch
// - Add it to Zbus server
// - If udev sees device removed then remove the zbus path

use std::collections::HashSet;
use std::sync::Arc;

use log::{debug, error, info, warn};
use mio::{Events, Interest, Poll, Token};
use rog_aura::aura_detection::LaptopLedData;
use rog_aura::usb::AuraDevice;
use rog_platform::hid_raw::HidRaw;
use tokio::sync::Mutex;
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
    pub async fn new(mut connection: Connection) -> Result<Self, RogError> {
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
            start_tasks(zbus, &mut connection, sig_ctx2, &path).await?;
        }

        // connection.object_server().at("/org/asuslinux",
        // ObjectManager).await.unwrap();

        let manager = Self {
            _connection: connection,
        };

        // detect all plugged in aura devices (eventually)
        let interfaces = Arc::new(Mutex::new(interfaces));
        let mut count = 0;
        tokio::spawn(async move {
            let mut monitor = MonitorBuilder::new()?.match_subsystem("hidraw")?.listen()?;
            let mut poll = Poll::new()?;
            let mut events = Events::with_capacity(1024);
            poll.registry()
                .register(&mut monitor, Token(0), Interest::READABLE)?;

            loop {
                if poll.poll(&mut events, None).is_err() {
                    continue;
                }
                // collect and sort so remove events are first
                // let mut events: Vec<udev::Event> = monitor.iter().filter(|e|
                // &*e.action().unwrap_or_default() == "remove").collect();
                // let mut adds: Vec<udev::Event> = monitor.iter().filter(|e|
                // &*e.action().unwrap_or_default() == "add").collect();
                // events.append(&mut adds);

                dbg!("LOOPED", count);
                count += 1;
                for event in monitor.iter() {
                    if event.parent_with_subsystem("hidraw").is_err() {
                        continue;
                    }

                    if let Some(parent) =
                        event.parent_with_subsystem_devtype("usb", "usb_device")?
                    {
                        let action = if let Some(action) = event.action() {
                            action
                        } else {
                            continue;
                        };

                        let id_product =
                            if let Some(id_product) = parent.attribute_value("idProduct") {
                                id_product.to_string_lossy()
                            } else {
                                continue;
                            };
                        let aura_device = AuraDevice::from(&*id_product);
                        if aura_device == AuraDevice::Unknown {
                            warn!("idProduct:{id_product:?} is unknown, not using");
                            continue;
                        }

                        let path = if let Some(path) = dbus_path_for_dev(&parent) {
                            path
                        } else {
                            continue;
                        };

                        dbg!(action, &aura_device, &path);
                        if action == "remove" {
                            info!("AuraManager removing: {path:?}");
                            let conn_copy = conn_copy.clone();
                            let interfaces_copy = interfaces.clone();
                            tokio::spawn(async move {
                                let mut interfaces = interfaces_copy.lock().await; // hold until completed
                                dbg!(&interfaces);
                                if interfaces.remove(&path) {
                                    let res = conn_copy
                                        .object_server()
                                        .remove::<CtrlAuraZbus, _>(&path)
                                        .await
                                        .map_err(|e| {
                                            error!("Failed to remove {path:?}, {e:?}");
                                            e
                                        })?;
                                    info!("AuraManager removed: {path:?}, {res}");
                                }
                                dbg!(&interfaces);
                                Ok::<(), RogError>(())
                            });
                        } else if action == "add" {
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

                            let path = if let Some(path) = dbus_path_for_dev(&parent) {
                                path
                            } else {
                                continue;
                            };

                            let dev_node = if let Some(dev_node) = event.devnode() {
                                dev_node.to_owned()
                            } else {
                                continue;
                            };

                            if let Ok(raw) = HidRaw::from_device(event.device())
                                .map_err(|e| error!("device path error: {e:?}"))
                            {
                                // bah... shitty clone TODO: fix
                                let data_clone = data.clone();
                                let mut conn_copy = conn_copy.clone();
                                let interfaces_copy = interfaces.clone();
                                //
                                tokio::spawn(async move {
                                    let mut interfaces = interfaces_copy.lock().await;
                                    dbg!(&interfaces);
                                    if interfaces.contains(&path) {
                                        debug!("Already a ctrl at {path:?}");
                                        return Ok(());
                                    }
                                    if let Ok(mut ctrl) =
                                        CtrlKbdLed::from_hidraw(raw, path.clone(), &data_clone)
                                    {
                                        info!(
                                            "AuraManager found device at: {dev_node:?}, {path:?}"
                                        );
                                        debug!("Starting Aura at {path}");
                                        interfaces.insert(path.clone());
                                        let sig_ctx = CtrlAuraZbus::signal_context(&conn_copy)?;
                                        ctrl.config =
                                            CtrlKbdLed::init_config(aura_device, &data_clone);
                                        let zbus = CtrlAuraZbus::new(ctrl, sig_ctx);
                                        // Now add it to device list
                                        let sig_ctx = CtrlAuraZbus::signal_context(&conn_copy)?;
                                        start_tasks(zbus, &mut conn_copy, sig_ctx, &path).await?;
                                    }
                                    dbg!(&interfaces);
                                    Ok::<(), RogError>(())
                                }); // Can't get result from here due to
                                    // MonitorSocket
                            }
                        }
                    }
                }
            }
            // Required for return type on tokio::spawn
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
    connection: &mut Connection,
    signal_ctx: SignalContext<'static>,
    path: &ObjectPath<'static>,
) -> Result<(), RogError> {
    let task = zbus.clone();
    zbus.reload()
        .await
        .unwrap_or_else(|err| warn!("Controller error: {}", err));

    connection
        .object_server()
        .at(&ObjectPath::from_str_unchecked(path), zbus)
        .await
        .unwrap();
    task.create_tasks(signal_ctx).await.ok();
    Ok(())
}
