pub mod setup_anime;
pub mod setup_aura;
pub mod setup_fans;
pub mod setup_system;

use std::sync::{Arc, Mutex};

use config_traits::StdConfig;
use rog_dbus::zbus_platform::PlatformProxyBlocking;
use slint::{ComponentHandle, PhysicalSize, SharedString, Weak};

use crate::config::Config;
use crate::ui::setup_anime::setup_anime_page;
use crate::ui::setup_aura::{has_aura_iface_blocking, setup_aura_page};
use crate::ui::setup_fans::setup_fan_curve_page;
use crate::ui::setup_system::{setup_system_page, setup_system_page_callbacks};
use crate::{AppSettingsPageData, MainWindow};

// This macro expects are consistent naming between proxy calls and slint
// globals
#[macro_export]
macro_rules! set_ui_props_async {
    ($ui:ident, $proxy:ident, $global:ident, $proxy_fn:ident) => {
        if let Ok(value) = $proxy.$proxy_fn().await {
            $ui.upgrade_in_event_loop(move |handle| {
                concat_idents::concat_idents!(set = set_, $proxy_fn {
                    handle.global::<$global>().set(value.into());
                });
            }).ok();
        }
    };
}

// this macro sets up:
// - a link from UI callback -> dbus proxy property
// - a link from dbus property signal -> UI state
// conv1 and conv2 are type conversion args
#[macro_export]
macro_rules! set_ui_callbacks {
    ($handle:ident, $data:ident($($conv1: tt)*),$proxy:ident.$proxy_fn:tt($($conv2: tt)*),$success:literal,$failed:literal) => {
        let handle_copy = $handle.as_weak();
        let proxy_copy = $proxy.clone();
        let data = $handle.global::<$data>();
        concat_idents::concat_idents!(on_set = on_set_, $proxy_fn {
        data.on_set(move |value| {
            let proxy_copy = proxy_copy.clone();
            let handle_copy = handle_copy.clone();
            tokio::spawn(async move {
                concat_idents::concat_idents!(set = set_, $proxy_fn {
                show_toast(
                    format!($success, value).into(),
                    $failed.into(),
                    handle_copy,
                    proxy_copy.set(value $($conv2)*).await,
                );
                });
            });
            });
        });
        let handle_copy = $handle.as_weak();
        let proxy_copy = $proxy.clone();
        concat_idents::concat_idents!(receive = receive_, $proxy_fn, _changed {
        // spawn required since the while let never exits
        tokio::spawn(async move {
            let mut x = proxy_copy.receive().await;
            concat_idents::concat_idents!(set = set_, $proxy_fn {
            use zbus::export::futures_util::StreamExt;
            while let Some(e) = x.next().await {
                if let Ok(out) = e.get().await {
                    handle_copy.upgrade_in_event_loop(move |handle| {
                        handle.global::<$data>().set(out $($conv1)*);
                    }).ok();
                }
            }
            });
        });
        });
    };
}

pub fn show_toast(
    success: SharedString,
    fail: SharedString,
    handle: Weak<MainWindow>,
    result: zbus::Result<()>,
) {
    match result {
        Ok(_) => {
            slint::invoke_from_event_loop(move || handle.unwrap().invoke_show_toast(success)).ok()
        }
        Err(e) => slint::invoke_from_event_loop(move || {
            log::warn!("{fail}: {e}");
            handle.unwrap().invoke_show_toast(fail)
        })
        .ok(),
    };
}

pub fn setup_window(config: Arc<Mutex<Config>>) -> MainWindow {
    let ui = MainWindow::new().unwrap();
    if let Ok(lock) = config.try_lock() {
        let fullscreen = lock.start_fullscreen;
        let width = lock.fullscreen_width;
        let height = lock.fullscreen_height;
        if fullscreen {
            ui.window().set_fullscreen(fullscreen);
            ui.window().set_size(PhysicalSize { width, height });
        }
    };

    let conn = zbus::blocking::Connection::system().unwrap();
    let platform = PlatformProxyBlocking::new(&conn).unwrap();

    let interfaces = platform.supported_interfaces().unwrap();
    log::debug!("Available interfaces: {interfaces:?}");
    // "Anime", "Aura", "FanCurves", "Platform"
    ui.set_sidebar_items_avilable(
        [
            // Needs to match the order of slint sidebar items
            interfaces.contains(&"Platform".into()),
            has_aura_iface_blocking().unwrap_or(false),
            interfaces.contains(&"Anime".into()),
            interfaces.contains(&"FanCurves".into()),
            true,
            true,
        ]
        .into(),
    );

    ui.on_exit_app(move || {
        slint::quit_event_loop().unwrap();
    });

    setup_app_settings_page(&ui, config.clone());
    setup_system_page(&ui, config.clone());
    setup_system_page_callbacks(&ui, config.clone());
    setup_aura_page(&ui, config.clone());
    setup_anime_page(&ui, config.clone());
    setup_fan_curve_page(&ui, config);
    ui
}

pub fn setup_app_settings_page(ui: &MainWindow, config: Arc<Mutex<Config>>) {
    let config_copy = config.clone();
    let global = ui.global::<AppSettingsPageData>();
    global.on_set_run_in_background(move |enable| {
        if let Ok(mut lock) = config_copy.try_lock() {
            lock.run_in_background = enable;
            lock.write();
        }
    });
    let config_copy = config.clone();
    global.on_set_startup_in_background(move |enable| {
        if let Ok(mut lock) = config_copy.try_lock() {
            lock.startup_in_background = enable;
            lock.write();
        }
    });
    let config_copy = config.clone();
    global.on_set_enable_tray_icon(move |enable| {
        if let Ok(mut lock) = config_copy.try_lock() {
            lock.enable_tray_icon = enable;
            lock.write();
        }
    });
    let config_copy = config.clone();
    global.on_set_enable_notifications(move |enable| {
        if let Ok(mut lock) = config_copy.try_lock() {
            lock.enable_notifications = enable;
            lock.write();
        }
    });

    if let Ok(lock) = config.try_lock() {
        global.set_run_in_background(lock.run_in_background);
        global.set_startup_in_background(lock.startup_in_background);
        global.set_enable_tray_icon(lock.enable_tray_icon);
        global.set_enable_notifications(lock.enable_notifications);
    }
}
