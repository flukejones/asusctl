use crate::config::Config;
use dbus::tree::{Factory, MTSync, Method, MethodErr, Signal, Tree};
use log::warn;
use asus_nb::{aura_modes::AuraModes, DBUS_IFACE, DBUS_PATH};
use std::sync::Arc;
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};

fn set_keyboard_backlight(sender: Mutex<Sender<AuraModes>>) -> Method<MTSync, ()> {
    let factory = Factory::new_sync::<()>();
    factory
        // method for ledmessage
        .method("SetKeyBacklight", (), {
            move |m| {
                let json: &str = m.msg.read1()?;
                if let Ok(mut lock) = sender.try_lock() {
                    if let Ok(data) = serde_json::from_str(json) {
                        lock.try_send(data).unwrap_or_else(|err| {
                            warn!("SetKeyBacklight over mpsc failed: {}", err)
                        });
                    } else {
                        warn!("SetKeyBacklight could not deserialise");
                    }
                    Ok(vec![])
                } else {
                    Err(MethodErr::failed("Could not lock daemon for access"))
                }
            }
        })
        .inarg::<&str, _>("json")
        .annotate("org.freedesktop.DBus.Method.NoReply", "true")
}

fn get_keyboard_backlight(config: Arc<Mutex<Config>>) -> Method<MTSync, ()> {
    let factory = Factory::new_sync::<()>();
    factory
        .method("GetKeyBacklight", (), {
            move |m| {
                if let Ok(lock) = config.try_lock() {
                    for mode in &lock.kbd_backlight_modes {
                        if lock.kbd_backlight_mode == <u8>::from(mode) {
                            let mode = serde_json::to_string(&mode).unwrap();
                            let mret = m.msg.method_return().append1(mode);
                            return Ok(vec![mret]);
                        }
                    }
                    Err(MethodErr::failed(
                        "Keyboard LED mode set to an invalid mode",
                    ))
                } else {
                    Err(MethodErr::failed("Could not lock config for access"))
                }
            }
        })
        .outarg::<&str, _>("json")
}

fn get_keyboard_backlight_modes(config: Arc<Mutex<Config>>) -> Method<MTSync, ()> {
    let factory = Factory::new_sync::<()>();
    factory
        .method("GetKeyBacklightModes", (), {
            move |m| {
                if let Ok(lock) = config.try_lock() {
                    let mode = serde_json::to_string(&lock.kbd_backlight_modes).unwrap();
                    let mret = m.msg.method_return().append1(mode);
                    Ok(vec![mret])
                } else {
                    Err(MethodErr::failed("Could not lock config for access"))
                }
            }
        })
        .outarg::<&str, _>("json")
}

fn set_animatrix(
    sender: Mutex<Sender<Vec<Vec<u8>>>>, // need mutex only to get interior mutability in MTSync
) -> Method<MTSync, ()> {
    let factory = Factory::new_sync::<()>();
    factory
        // method for ledmessage
        .method("AnimatrixWrite", (), {
            move |m| {
                let mut iter = m.msg.iter_init();
                let byte_array: Vec<Vec<u8>> = vec![iter.read()?, iter.read()?];
                if let Ok(mut lock) = sender.try_lock() {
                    // Ignore errors if the channel is already full
                    lock.try_send(byte_array).unwrap_or_else(|_err| {});
                    Ok(vec![])
                } else {
                    Err(MethodErr::failed("Could not lock daemon for access"))
                }
            }
        })
        .inarg::<Vec<u8>, _>("bytearray1")
        .inarg::<Vec<u8>, _>("bytearray2")
        .annotate("org.freedesktop.DBus.Method.NoReply", "true")
}

fn set_fan_mode(sender: Mutex<Sender<u8>>) -> Method<MTSync, ()> {
    let factory = Factory::new_sync::<()>();
    factory
        // method for ledmessage
        .method("SetFanMode", (), {
            move |m| {
                if let Ok(mut lock) = sender.try_lock() {
                    let mut iter = m.msg.iter_init();
                    let byte: u8 = iter.read()?;
                    lock.try_send(byte).unwrap_or_else(|_err| {});
                    Ok(vec![])
                } else {
                    Err(MethodErr::failed("Could not lock daemon for access"))
                }
            }
        })
        .inarg::<u8, _>("mode")
        .annotate("org.freedesktop.DBus.Method.NoReply", "true")
}

fn get_fan_mode(config: Arc<Mutex<Config>>) -> Method<MTSync, ()> {
    let factory = Factory::new_sync::<()>();
    factory
        .method("GetFanMode", (), {
            move |m| {
                if let Ok(lock) = config.try_lock() {
                    let mret = m.msg.method_return().append1(lock.power_profile);
                    Ok(vec![mret])
                } else {
                    Err(MethodErr::failed("Could not lock config for access"))
                }
            }
        })
        .outarg::<u8, _>("mode")
}

fn get_charge_limit(config: Arc<Mutex<Config>>) -> Method<MTSync, ()> {
    let factory = Factory::new_sync::<()>();
    factory
        .method("GetChargeLimit", (), {
            move |m| {
                if let Ok(lock) = config.try_lock() {
                    let mret = m.msg.method_return().append1(lock.bat_charge_limit);
                    Ok(vec![mret])
                } else {
                    Err(MethodErr::failed("Could not lock config for access"))
                }
            }
        })
        .outarg::<u8, _>("limit")
}

fn set_charge_limit(sender: Mutex<Sender<u8>>) -> Method<MTSync, ()> {
    let factory = Factory::new_sync::<()>();
    factory
        // method for ledmessage
        .method("SetChargeLimit", (), {
            move |m| {
                if let Ok(mut lock) = sender.try_lock() {
                    let mut iter = m.msg.iter_init();
                    let byte: u8 = iter.read()?;
                    lock.try_send(byte).unwrap_or_else(|_err| {});
                    Ok(vec![])
                } else {
                    Err(MethodErr::failed("Could not lock daemon for access"))
                }
            }
        })
        .inarg::<u8, _>("limit")
        .annotate("org.freedesktop.DBus.Method.NoReply", "true")
}

#[allow(clippy::type_complexity)]
pub fn dbus_create_tree(
    config: Arc<Mutex<Config>>,
) -> (
    Tree<MTSync, ()>,
    Receiver<AuraModes>,
    Receiver<Vec<Vec<u8>>>,
    Receiver<u8>,
    Receiver<u8>,
    Arc<Signal<()>>,
    Arc<Signal<()>>,
    Arc<Signal<()>>,
) {
    let (aura_command_send, aura_command_recv) = channel::<AuraModes>(1);
    let (animatrix_send, animatrix_recv) = channel::<Vec<Vec<u8>>>(1);
    let (fan_mode_send, fan_mode_recv) = channel::<u8>(1);
    let (charge_send, charge_recv) = channel::<u8>(1);

    let factory = Factory::new_sync::<()>();

    let key_backlight_changed = Arc::new(
        factory
            .signal("KeyBacklightChanged", ())
            .sarg::<&str, _>("json"),
    );
    let chrg_limit_changed = Arc::new(
        factory
            .signal("ChargeLimitChanged", ())
            .sarg::<u8, _>("limit"),
    );
    let fanmode_changed = Arc::new(factory.signal("FanModeChanged", ()).sarg::<u8, _>("mode"));

    let tree = factory
        .tree(())
        .add(
            factory.object_path(DBUS_PATH, ()).introspectable().add(
                factory
                    .interface(DBUS_IFACE, ())
                    .add_m(set_keyboard_backlight(Mutex::new(aura_command_send)))
                    .add_m(set_animatrix(Mutex::new(animatrix_send)))
                    .add_m(set_fan_mode(Mutex::new(fan_mode_send)))
                    .add_m(set_charge_limit(Mutex::new(charge_send)))
                    .add_m(get_fan_mode(config.clone()))
                    .add_m(get_charge_limit(config.clone()))
                    .add_m(get_keyboard_backlight(config.clone()))
                    .add_m(get_keyboard_backlight_modes(config))
                    .add_s(key_backlight_changed.clone())
                    .add_s(fanmode_changed.clone())
                    .add_s(chrg_limit_changed.clone()),
            ),
        )
        .add(factory.object_path("/", ()).introspectable());
    (
        tree,
        aura_command_recv,
        animatrix_recv,
        fan_mode_recv,
        charge_recv,
        key_backlight_changed,
        fanmode_changed,
        chrg_limit_changed,
    )
}
