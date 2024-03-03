//! A seld-contained tray icon with menus. The control of app<->tray is done via
//! commands over an MPSC channel.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use betrayer::{Icon, Menu, MenuItem, TrayEvent, TrayIconBuilder};
use log::{debug, error, info, warn};
use rog_platform::platform::{GpuMode, Properties};
use supergfxctl::pci_device::{GfxMode, GfxPower};
use supergfxctl::zbus_proxy::DaemonProxyBlocking as GfxProxy;
use versions::Versioning;

use crate::config::Config;
use crate::system_state::SystemState;
use crate::{get_ipc_file, QUIT_APP, SHOW_GUI};

const TRAY_LABEL: &str = "ROG Control Center";
const TRAY_ICON_PATH: &str = "/usr/share/icons/hicolor/512x512/apps/";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TrayAction {
    Open,
    Quit,
}

fn open_app() {
    if let Ok(mut ipc) = get_ipc_file().map_err(|e| {
        error!("ROGTray: get_ipc_file: {}", e);
    }) {
        debug!("Tray told app to show self");
        ipc.write_all(&[SHOW_GUI, 0]).ok();
    }
}

fn quit_app() {
    if let Ok(mut ipc) = get_ipc_file().map_err(|e| {
        error!("ROGTray: get_ipc_file: {}", e);
    }) {
        debug!("Tray told app to show self");
        ipc.write_all(&[QUIT_APP, 0]).ok();
    }
}

fn read_icon(file: &Path) -> Icon {
    let mut path = PathBuf::from(TRAY_ICON_PATH);
    path.push(file);
    let mut file = OpenOptions::new().read(true).open(path).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();

    Icon::from_png_bytes(&bytes)
        .unwrap_or(Icon::from_rgba(vec![255u8; 32 * 32 * 4], 32, 32).unwrap())
}

fn build_menu() -> Menu<TrayAction> {
    Menu::new([
        MenuItem::separator(),
        MenuItem::button("Open", TrayAction::Open),
        MenuItem::button("Quit", TrayAction::Quit),
    ])
}

/// The tray is controlled somewhat by `Arc<Mutex<SystemState>>`
pub fn init_tray(
    _supported_properties: Vec<Properties>,
    states: Arc<Mutex<SystemState>>,
    config: Arc<Mutex<Config>>,
) {
    std::thread::spawn(move || {
        debug!("init_tray");

        let conn = zbus::blocking::Connection::system().unwrap();
        let gfx_proxy = GfxProxy::new(&conn).unwrap();
        let mut supergfx_active = false;
        if gfx_proxy.mode().is_ok() {
            supergfx_active = true;
            if let Ok(version) = gfx_proxy.version() {
                if let Some(version) = Versioning::new(&version) {
                    let curr_gfx = Versioning::new("5.0.3-RC4").unwrap();
                    warn!("supergfxd version = {version}");
                    if version < curr_gfx {
                        // Don't allow mode changing if too old a version
                        warn!("supergfxd found but is too old to use");
                        // tray.gfx_proxy_is_active = false;
                    }
                }
            }
        };

        let rog_blue = read_icon(&PathBuf::from("asus_notif_blue.png"));
        let rog_red = read_icon(&PathBuf::from("asus_notif_red.png"));
        let rog_green = read_icon(&PathBuf::from("asus_notif_green.png"));
        let rog_white = read_icon(&PathBuf::from("asus_notif_white.png"));
        let gpu_integrated = read_icon(&PathBuf::from("rog-control-center.png"));

        info!("Started ROGTray");

        let tray = TrayIconBuilder::<TrayAction>::new()
            .with_icon(rog_red.clone())
            .with_tooltip(TRAY_LABEL)
            .with_menu(build_menu())
            .build(|event| {
                if let TrayEvent::Menu(action) = event {
                    match action {
                        TrayAction::Open => open_app(),
                        TrayAction::Quit => {
                            quit_app();
                            exit(0);
                        }
                    }
                }
            })
            .unwrap();

        loop {
            // let states = states.clone();
            if let Ok(mut lock) = states.lock() {
                if lock.tray_should_update {
                    // Supergfx ends up adding some complexity to handle if it isn't available
                    let current_gpu_mode = if supergfx_active {
                        lock.gfx_state.mode
                    } else if let Some(mode) = lock.bios.gpu_mux_mode {
                        match mode {
                            GpuMode::Discrete => GfxMode::AsusMuxDgpu,
                            _ => GfxMode::Hybrid,
                        }
                    } else {
                        GfxMode::Hybrid
                    };
                    dbg!(current_gpu_mode);
                    dbg!(lock.bios.gpu_mux_mode);
                    tray.set_tooltip(format!("ROG: gpu mode = {current_gpu_mode:?}"));

                    lock.tray_should_update = false;
                    debug!("ROGTray: rebuilt menus due to state change");

                    match lock.gfx_state.power_status {
                        GfxPower::Suspended => tray.set_icon(Some(rog_blue.clone())),
                        GfxPower::Off => {
                            if lock.gfx_state.mode == GfxMode::Vfio {
                                tray.set_icon(Some(rog_red.clone()))
                            } else {
                                tray.set_icon(Some(rog_green.clone()))
                            }
                        }
                        GfxPower::AsusDisabled => tray.set_icon(Some(rog_white.clone())),
                        GfxPower::AsusMuxDiscreet | GfxPower::Active => {
                            tray.set_icon(Some(rog_red.clone()));
                        }
                        GfxPower::Unknown => {
                            if supergfx_active {
                                tray.set_icon(Some(gpu_integrated.clone()));
                            } else {
                                tray.set_icon(Some(rog_red.clone()));
                            }
                        }
                    };

                    if let Ok(lock) = config.try_lock() {
                        if !lock.enable_tray_icon {
                            return;
                        }
                    }
                }
            }
            sleep(Duration::from_millis(50));
        }
    });
}
