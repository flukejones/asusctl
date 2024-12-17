// Plan:
// - Manager has udev monitor on USB looking for ROG devices
// - If a device is found, add it to watch
// - Add it to Zbus server
// - If udev sees device removed then remove the zbus path

use std::collections::HashSet;

use log::{error, info, warn};
use mio::{Events, Interest, Poll, Token};
use udev::{Device, MonitorBuilder};
use zbus::object_server::SignalEmitter;
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
            _connection: connection,
        };

        // detect all plugged in aura devices (eventually)
        // only USB devices are detected for here
        std::thread::spawn(move || {
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

pub(crate) fn dbus_path_for_dev(parent: &Device) -> Option<OwnedObjectPath> {
    if let Some(filename) = super::filename_partial(parent) {
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
    _signal_ctx: SignalEmitter<'static>,
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
