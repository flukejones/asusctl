use std::sync::{Arc, Mutex};

use rog_aura::usb::AuraPowerDev;
use rog_dbus::zbus_aura::AuraProxy;
use slint::{ComponentHandle, Model, RgbaColor, SharedString};

use crate::config::Config;
use crate::ui::show_toast;
use crate::{
    set_ui_callbacks, set_ui_props_async, AuraPageData, MainWindow, PowerZones as SlintPowerZones,
};

fn decode_hex(s: &str) -> RgbaColor<u8> {
    let s = s.trim_start_matches('#');
    let c: Vec<u8> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or(164))
        .collect();
    RgbaColor {
        alpha: 255,
        red: *c.first().unwrap_or(&255),
        green: *c.get(1).unwrap_or(&128),
        blue: *c.get(2).unwrap_or(&32),
    }
}

pub fn has_aura_iface_blocking() -> Result<bool, Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::system()?;
    let f = zbus::blocking::fdo::ObjectManagerProxy::new(&conn, "org.asuslinux.Daemon", "/org")?;
    let interfaces = f.get_managed_objects()?;
    let mut aura_paths = Vec::new();
    for v in interfaces.iter() {
        for k in v.1.keys() {
            if k.as_str() == "org.asuslinux.Aura" {
                aura_paths.push(v.0.clone());
            }
        }
    }
    Ok(!aura_paths.is_empty())
}

/// Returns the first available Aura interface
// TODO: return all
async fn find_aura_iface() -> Result<AuraProxy<'static>, Box<dyn std::error::Error>> {
    let conn = zbus::Connection::system().await?;
    let f = zbus::fdo::ObjectManagerProxy::new(&conn, "org.asuslinux.Daemon", "/org").await?;
    let interfaces = f.get_managed_objects().await?;
    let mut aura_paths = Vec::new();
    for v in interfaces.iter() {
        for k in v.1.keys() {
            if k.as_str() == "org.asuslinux.Aura" {
                println!("Found aura device at {}, {}", v.0, k);
                aura_paths.push(v.0.clone());
            }
        }
    }
    if aura_paths.len() > 1 {
        println!("Multiple aura devices found: {aura_paths:?}");
        println!("TODO: enable selection");
    }
    if let Some(path) = aura_paths.first() {
        return Ok(AuraProxy::builder(&conn)
            .path(path.clone())?
            .destination("org.asuslinux.Daemon")?
            .build()
            .await?);
    }

    Err("No Aura interface".into())
}

pub fn setup_aura_page(ui: &MainWindow, _states: Arc<Mutex<Config>>) {
    ui.global::<AuraPageData>().on_set_hex_from_colour(|c| {
        format!("#{:02X}{:02X}{:02X}", c.red(), c.green(), c.blue()).into()
    });

    ui.global::<AuraPageData>()
        .on_set_hex_to_colour(|s| decode_hex(s.as_str()).into());

    let handle = ui.as_weak();
    tokio::spawn(async move {
        let aura = find_aura_iface().await.unwrap();

        set_ui_props_async!(handle, aura, AuraPageData, brightness);
        set_ui_props_async!(handle, aura, AuraPageData, led_mode);
        set_ui_props_async!(handle, aura, AuraPageData, led_mode_data);
        set_ui_props_async!(handle, aura, AuraPageData, led_power);

        if let Ok(power) = aura.supported_power_zones().await {
            log::debug!("Available LED power modes {power:?}");
            let power: Vec<SlintPowerZones> = power.iter().map(|p| (*p).into()).collect();
            handle
                .upgrade_in_event_loop(move |handle| {
                    handle
                        .global::<AuraPageData>()
                        .set_supported_power_zones(power.as_slice().into());
                })
                .ok();
        }

        if let Ok(modes) = aura.supported_basic_modes().await {
            log::debug!("Available LED modes {modes:?}");
            handle
                .upgrade_in_event_loop(move |handle| {
                    let m: Vec<i32> = modes.iter().map(|n| (*n).into()).collect();
                    handle
                        .global::<AuraPageData>()
                        .set_supported_basic_modes(m.as_slice().into());
                    // Get the translated names
                    let names = handle.global::<AuraPageData>().get_mode_names();

                    let res: Vec<SharedString> = names
                        .iter()
                        .enumerate()
                        .filter(|(n, _)| modes.contains(&(*n as i32).into()) && *n != 9)
                        .map(|(_, i)| i)
                        .collect();
                    handle
                        .global::<AuraPageData>()
                        .set_available_mode_names(res.as_slice().into());
                })
                .ok();
        }

        let proxy_copy = aura.clone();
        handle
            .upgrade_in_event_loop(move |handle| {
                set_ui_callbacks!(handle,
                    AuraPageData(.into()),
                    proxy_copy.brightness(.into()),
                    "Keyboard LED brightness successfully set to {}",
                    "Setting keyboard LED brightness failed"
                );

                set_ui_callbacks!(handle,
                    AuraPageData(.into()),
                    proxy_copy.led_mode(.into()),
                    "Keyboard LED mode successfully set to {}",
                    "Setting keyboard LEDmode failed"
                );

                set_ui_callbacks!(handle,
                    AuraPageData(.into()),
                    proxy_copy.led_mode_data(.into()),
                    "Keyboard LED mode set to {:?}",
                    "Setting keyboard LED mode failed"
                );

                // set_ui_callbacks!(handle,
                //     AuraPageData(.clone().into()),
                //     proxy_copy.led_power(.into()),
                //     "Keyboard LED power successfully set to {:?}",
                //     "Setting keyboard power failed"
                // );

                handle.invoke_external_colour_change();
            })
            .ok();

        let handle_copy = handle.clone();
        let proxy_copy = aura.clone();
        handle
            .upgrade_in_event_loop(|handle| {
                handle
                    .global::<AuraPageData>()
                    .on_set_led_power(move |power| {
                        let handle_copy = handle_copy.clone();
                        let proxy_copy = aura.clone();
                        let power: AuraPowerDev = power.into();
                        tokio::spawn(async move {
                            show_toast(
                                "Anime builtin animations changed".into(),
                                "Failed to set Anime builtin animations".into(),
                                handle_copy,
                                proxy_copy.set_led_power(power).await,
                            );
                        });
                    });
            })
            .unwrap();

        // Need to update the UI if the mode changes
        let handle_copy = handle.clone();
        // spawn required since the while let never exits
        tokio::spawn(async move {
            let mut x = proxy_copy.receive_led_mode_data_changed().await;
            use zbus::export::futures_util::StreamExt;
            while let Some(e) = x.next().await {
                if let Ok(out) = e.get().await {
                    handle_copy
                        .upgrade_in_event_loop(move |handle| {
                            handle
                                .global::<AuraPageData>()
                                .invoke_update_led_mode_data(out.into());
                            handle.invoke_external_colour_change();
                        })
                        .ok();
                }
            }
        });
    });
}
