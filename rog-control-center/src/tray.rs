//! A self-contained tray icon with menus.

use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use ksni::{Handle, Icon, TrayMethods};
use log::{info, warn};
use rog_platform::platform::Properties;
use supergfxctl::pci_device::{Device, GfxMode, GfxPower};
use supergfxctl::zbus_proxy::DaemonProxy as GfxProxy;
use versions::Versioning;

use crate::config::Config;
use crate::zbus_proxies::{AppState, ROGCCZbusProxyBlocking};

const TRAY_LABEL: &str = "ROG Control Center";
const TRAY_ICON_PATH: &str = "/usr/share/icons/hicolor/512x512/apps/";

struct Icons {
    rog_blue: Icon,
    rog_red: Icon,
    rog_green: Icon,
    rog_white: Icon,
    gpu_integrated: Icon
}

static ICONS: OnceLock<Icons> = OnceLock::new();

fn read_icon(file: &Path) -> Icon {
    let mut path = PathBuf::from(TRAY_ICON_PATH);
    path.push(file);
    let mut file = OpenOptions::new().read(true).open(path).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();

    let mut img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
        .expect("icon not found")
        .to_rgba8();
    for image::Rgba(pixel) in img.pixels_mut() {
        // (╯°□°）╯︵ ┻━┻
        *pixel = u32::from_be_bytes(*pixel).rotate_right(8).to_be_bytes();
    }

    let (width, height) = img.dimensions();
    Icon {
        width: width as i32,
        height: height as i32,
        data: img.into_raw()
    }
}

struct AsusTray {
    current_title: String,
    current_icon: Icon,
    proxy: ROGCCZbusProxyBlocking<'static>
}

impl ksni::Tray for AsusTray {
    fn id(&self) -> String {
        TRAY_LABEL.into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![self.current_icon.clone()]
    }

    fn title(&self) -> String {
        self.current_title.clone()
    }

    fn status(&self) -> ksni::Status {
        ksni::Status::Active
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Open ROGCC".into(),
                icon_name: "rog-control-center".into(),
                activate: Box::new(move |s: &mut AsusTray| {
                    s.proxy.set_state(AppState::MainWindowShouldOpen).ok();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit ROGCC".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

async fn set_tray_icon_and_tip(
    mode: GfxMode,
    power: GfxPower,
    tray: &mut Handle<AsusTray>,
    supergfx_active: bool
) {
    if let Some(icons) = ICONS.get() {
        let icon = match power {
            GfxPower::Suspended => icons.rog_blue.clone(),
            GfxPower::Off => {
                if mode == GfxMode::Vfio {
                    icons.rog_red.clone()
                } else {
                    icons.rog_green.clone()
                }
            }
            GfxPower::AsusDisabled => icons.rog_white.clone(),
            GfxPower::AsusMuxDiscreet | GfxPower::Active => icons.rog_red.clone(),
            GfxPower::Unknown => {
                if supergfx_active {
                    icons.gpu_integrated.clone()
                } else {
                    icons.rog_red.clone()
                }
            }
        };

        tray.update(|tray: &mut AsusTray| {
            tray.current_icon = icon;
            tray.current_title = format!(
                "ROG: gpu mode = {mode:?}, gpu power =
            {power:?}"
            );
        })
        .await;
    }
}

fn find_dgpu() -> Option<Device> {
    use supergfxctl::pci_device::Device;
    let dev = Device::find().unwrap_or_default();
    for dev in dev {
        if dev.is_dgpu() {
            info!("Found dGPU: {}", dev.pci_id());
            // Plain old thread is perfectly fine since most of this is potentially blocking
            return Some(dev);
        }
    }
    warn!("Did not find a dGPU on this system, dGPU status won't be avilable");
    None
}

/// The tray is controlled somewhat by `Arc<Mutex<SystemState>>`
pub fn init_tray(_supported_properties: Vec<Properties>, config: Arc<Mutex<Config>>) {
    tokio::spawn(async move {
        let user_con = zbus::blocking::Connection::session().unwrap();
        let proxy = ROGCCZbusProxyBlocking::new(&user_con).unwrap();

        let rog_red = read_icon(&PathBuf::from("asus_notif_red.png"));

        let tray = AsusTray {
            current_title: TRAY_LABEL.to_string(),
            current_icon: rog_red.clone(),
            proxy
        };

        let mut tray = tray
            .spawn_without_dbus_name()
            .await
            .map_err(|e| {
                log::error!(
                    "Tray unable to be initialised: {e:?}. Do you have a system tray enabled?"
                )
            })
            .unwrap();

        info!("Tray started");
        let rog_blue = read_icon(&PathBuf::from("asus_notif_blue.png"));
        let rog_green = read_icon(&PathBuf::from("asus_notif_green.png"));
        let rog_white = read_icon(&PathBuf::from("asus_notif_white.png"));
        let gpu_integrated = read_icon(&PathBuf::from("rog-control-center.png"));
        ICONS.get_or_init(|| Icons {
            rog_blue,
            rog_red: rog_red.clone(),
            rog_green,
            rog_white,
            gpu_integrated
        });

        let mut has_supergfx = false;
        let conn = zbus::Connection::system().await.unwrap();
        if let Ok(gfx_proxy) = GfxProxy::new(&conn).await {
            match gfx_proxy.mode().await {
                Ok(_) => {
                    has_supergfx = true;
                    if let Ok(version) = gfx_proxy.version().await {
                        if let Some(version) = Versioning::new(&version) {
                            let curr_gfx = Versioning::new("5.2.0").unwrap();
                            warn!("supergfxd version = {version}");
                            if version < curr_gfx {
                                // Don't allow mode changing if too old a version
                                warn!("supergfxd found but is too old to use");
                                has_supergfx = false;
                            }
                        }
                    }
                }
                Err(e) => warn!("Couldn't get mode form supergfxd: {e:?}")
            }

            info!("Started ROGTray");
            let mut last_power = GfxPower::Unknown;
            let dev = find_dgpu();
            loop {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                if let Ok(lock) = config.try_lock() {
                    if !lock.enable_tray_icon {
                        return;
                    }
                }
                if has_supergfx {
                    if let Ok(mode) = gfx_proxy.mode().await {
                        if let Ok(power) = gfx_proxy.power().await {
                            if last_power != power {
                                set_tray_icon_and_tip(mode, power, &mut tray, has_supergfx).await;
                                last_power = power;
                            }
                        }
                    }
                } else if let Some(dev) = dev.as_ref() {
                    if let Ok(power) = dev.get_runtime_status() {
                        if last_power != power {
                            set_tray_icon_and_tip(GfxMode::Hybrid, power, &mut tray, has_supergfx)
                                .await;
                            last_power = power;
                        }
                    }
                }
            }
        }
    });
}
