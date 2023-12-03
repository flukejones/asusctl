//! A seld-contained tray icon with menus. The control of app<->tray is done via
//! commands over an MPSC channel.

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gtk::gio::Icon;
use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use log::{debug, error, info, trace, warn};
use rog_dbus::zbus_platform::PlatformProxyBlocking;
use rog_platform::platform::{GpuMode, Properties};
use supergfxctl::actions::UserActionRequired as GfxUserActionRequired;
use supergfxctl::pci_device::{GfxMode, GfxPower};
use supergfxctl::zbus_proxy::DaemonProxyBlocking as GfxProxyBlocking;
use versions::Versioning;

use crate::error::Result;
use crate::system_state::SystemState;
use crate::{get_ipc_file, SHOW_GUI};

const TRAY_APP_ICON: &str = "rog-control-center";
const TRAY_LABEL: &str = "ROG Control Center";

pub enum AppToTray {
    DgpuStatus(GfxPower),
}

pub struct RadioGroup(Vec<gtk::RadioMenuItem>);

impl RadioGroup {
    /// Add new radio menu item. `set_no_show_all()` is true until added to menu
    /// to prevent teh callback from running
    pub fn new<F>(first_label: &str, cb: F) -> Self
    where
        F: Fn(&gtk::RadioMenuItem) + Send + 'static,
    {
        let item = gtk::RadioMenuItem::with_label(first_label);
        item.set_active(false);
        item.set_no_show_all(true);
        item.connect_activate(move |this| {
            if this.is_active() && !this.is_no_show_all() {
                cb(this);
            }
        });
        Self(vec![item])
    }

    /// Add new radio menu item. `set_no_show_all()` is true until added to menu
    /// to prevent teh callback from running
    pub fn add<F>(&mut self, label: &str, cb: F)
    where
        F: Fn(&gtk::RadioMenuItem) + Send + 'static,
    {
        debug_assert!(!self.0.is_empty());
        let group = self.0[0].group();

        let item = gtk::RadioMenuItem::with_label_from_widget(&group[0], Some(label));
        item.set_active(false);
        item.set_no_show_all(true);
        item.connect_activate(move |this| {
            if this.is_active() && !this.is_no_show_all() {
                cb(this);
            }
        });
        self.0.push(item);
    }
}

pub struct ROGTray {
    tray: AppIndicator,
    menu: gtk::Menu,
    icon: &'static str,
    bios_proxy: PlatformProxyBlocking<'static>,
    gfx_proxy_is_active: bool,
    gfx_action: Arc<Mutex<GfxUserActionRequired>>,
    gfx_proxy: GfxProxyBlocking<'static>,
    states: Arc<Mutex<SystemState>>,
}

impl ROGTray {
    pub fn new(states: Arc<Mutex<SystemState>>) -> Result<Self> {
        let conn = zbus::blocking::Connection::system().map_err(|e| {
            error!("ROGTray: {e}");
            e
        })?;

        let gfx_proxy = GfxProxyBlocking::new(&conn).map_err(|e| {
            error!("ROGTray: {e}");
            e
        })?;

        let rog_tray = Self {
            tray: AppIndicator::new(TRAY_LABEL, TRAY_APP_ICON),
            menu: gtk::Menu::new(),
            icon: TRAY_APP_ICON,
            bios_proxy: PlatformProxyBlocking::new(&conn).map_err(|e| {
                error!("ROGTray: {e}");
                e
            })?,
            gfx_proxy_is_active: gfx_proxy.mode().is_ok(),
            gfx_action: Arc::new(Mutex::new(GfxUserActionRequired::Nothing)),
            gfx_proxy,
            states,
        };
        Ok(rog_tray)
    }

    pub fn set_icon(&mut self, icon: &'static str) {
        self.icon = icon;
        self.tray.set_icon(self.icon);
        self.tray.set_status(AppIndicatorStatus::Active);
    }

    /// Add a non-interactive label
    fn add_inactive_label(&mut self, label: &str) {
        let item = gtk::MenuItem::with_label(label);
        item.set_sensitive(false);
        self.menu.append(&item);
        self.menu.show_all();
    }

    /// Add a separator
    fn add_separator(&mut self) {
        let item = gtk::SeparatorMenuItem::new();
        item.set_sensitive(false);
        self.menu.append(&item);
        self.menu.show_all();
    }

    fn add_radio_sub_menu(
        &mut self,
        header_label: &str,
        active_label: &str,
        sub_menu: &RadioGroup,
    ) {
        let header_item = gtk::MenuItem::with_label(header_label);
        header_item.show_all();
        self.menu.add(&header_item);

        let menu = gtk::Menu::new();
        for item in &sub_menu.0 {
            if let Some(label) = item.label() {
                item.set_active(label == active_label);
            } else {
                item.set_active(false);
            }
            item.set_no_show_all(false);
            item.show_all();
            menu.add(item);
        }
        menu.show_all();
        header_item.set_submenu(Some(&menu));
    }

    fn _add_menu_item<F>(&mut self, label: &str, cb: F)
    where
        F: Fn() + Send + 'static,
    {
        let item = gtk::MenuItem::with_label(label);
        item.connect_activate(move |_| {
            cb();
        });
        self.menu.append(&item);
        self.menu.show_all();
    }

    fn add_check_menu_item<F>(&mut self, label: &str, is_active: bool, cb: F)
    where
        F: Fn(&gtk::CheckMenuItem) + Send + 'static,
    {
        let item = gtk::CheckMenuItem::with_label(label);
        item.set_active(is_active);
        item.connect_activate(move |this| {
            cb(this);
        });
        self.menu.append(&item);
        self.menu.show_all();
    }

    /// Add a menu item with an icon to the right
    fn add_icon_menu_item<F>(&mut self, label: &str, icon: &str, cb: F)
    where
        F: Fn() + Send + 'static,
    {
        let g_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let icon = gtk::Image::from_gicon(&Icon::for_string(icon).unwrap(), gtk::IconSize::Menu);
        let label = gtk::Label::new(Some(label));
        let item = gtk::MenuItem::new();
        g_box.add(&icon);
        g_box.add(&label);

        item.add(&g_box);
        item.show_all();

        item.connect_activate(move |_| {
            cb();
        });
        self.menu.append(&item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);
    }

    fn _set_status(&mut self, status: AppIndicatorStatus) {
        self.tray.set_status(status);
    }

    fn menu_add_base(&mut self) {
        self.add_icon_menu_item("Open app", "asus_notif_red", move || {
            if let Ok(mut ipc) = get_ipc_file().map_err(|e| {
                error!("ROGTray: get_ipc_file: {}", e);
            }) {
                debug!("Tray told app to show self");
                ipc.write_all(&[SHOW_GUI]).ok();
            }
        });

        self.add_separator();
        debug!("ROGTray: built base menu");
    }

    fn menu_add_charge_limit(&mut self, limit: u8) {
        self.add_inactive_label(&format!("Charge limit: {limit}"));
        debug!("ROGTray: appended charge limit menu");
    }

    fn menu_add_panel_od(&mut self, panel_od: bool) {
        let bios = self.bios_proxy.clone();
        let states = self.states.clone();
        self.add_check_menu_item("Panel Overdrive", panel_od, move |this| {
            if let Ok(mut lock) = states.lock() {
                lock.tray_should_update = true;
            }
            bios.set_panel_od(this.is_active())
                .map_err(|e| {
                    error!("ROGTray: set_panel_od: {e}");
                    e
                })
                .ok();
        });
        debug!("ROGTray: appended panel overdrive menu");
    }

    fn menu_add_mini_led_mode(&mut self, on: bool) {
        let bios = self.bios_proxy.clone();
        let states = self.states.clone();
        self.add_check_menu_item("MiniLED mode", on, move |this| {
            if let Ok(mut lock) = states.lock() {
                lock.tray_should_update = true;
            }
            bios.set_mini_led_mode(this.is_active())
                .map_err(|e| {
                    error!("ROGTray: set_mini_led_mode: {e}");
                    e
                })
                .ok();
        });
        debug!("ROGTray: appended miniLED mode menu");
    }

    fn menu_add_supergfx(&mut self, supported_gfx: &[GfxMode], current_mode: GfxMode) {
        if !self.gfx_proxy_is_active {
            trace!("menu_add_supergfx: gfx_proxy_is_active is false");
            return;
        }

        let gfx_dbus = self.gfx_proxy.clone();
        let gfx_action = self.gfx_action.clone();
        let states = self.states.clone();
        let mut gpu_menu = RadioGroup::new("Integrated", move |_| {
            if current_mode != GfxMode::Integrated {
                if let Ok(mut lock) = states.lock() {
                    lock.tray_should_update = true;
                }
                if let Ok(res) = gfx_dbus.set_mode(&GfxMode::Integrated).map_err(|e| {
                    error!("ROGTray: srt_mode: {e}");
                    e
                }) {
                    if let Ok(mut lock) = gfx_action.lock() {
                        *lock = res;
                    }
                }
            }
        });

        let mut func = |menu_mode: GfxMode| {
            let gfx_dbus = self.gfx_proxy.clone();
            let gfx_action = self.gfx_action.clone();
            let states = self.states.clone();
            gpu_menu.add(&format!("{menu_mode}"), move |_| {
                if current_mode != menu_mode {
                    if let Ok(mut lock) = states.lock() {
                        lock.tray_should_update = true;
                    }
                    if let Ok(res) = gfx_dbus.set_mode(&menu_mode).map_err(|e| {
                        error!("ROGTray: set_mode: {e}");
                        e
                    }) {
                        if let Ok(mut lock) = gfx_action.lock() {
                            *lock = res;
                        }
                    }
                }
            });
        };

        for item in supported_gfx {
            if *item == GfxMode::Integrated {
                continue;
            }
            func(*item);
        }

        let action_required = if let Ok(lock) = self.gfx_action.lock() {
            if matches!(*lock, GfxUserActionRequired::Nothing) {
                ""
            } else {
                <&str>::from(*lock)
            }
        } else {
            ""
        };

        self.add_radio_sub_menu(
            &format!("GPU Mode: {current_mode} {action_required}"),
            &current_mode.to_string(),
            &gpu_menu,
        );

        debug!("ROGTray: appended gpu menu");
    }

    fn menu_add_mux(&mut self, current_mode: GfxMode) {
        let gfx_dbus = self.bios_proxy.clone();

        let mut reboot_required = false;
        if let Ok(mode) = gfx_dbus.gpu_mux_mode() {
            let mode = match GpuMode::from(mode) {
                GpuMode::Discrete => GfxMode::AsusMuxDgpu,
                _ => GfxMode::Hybrid,
            };
            reboot_required = mode != current_mode;
        }

        let states = self.states.clone();
        let mut gpu_menu = RadioGroup::new("Optimus", move |_| {
            if let Ok(mut lock) = states.lock() {
                lock.tray_should_update = true;
            }
            gfx_dbus
                .set_gpu_mux_mode(GpuMode::Optimus)
                .map_err(|e| {
                    error!("ROGTray: set_mode: {e}");
                    e
                })
                .ok();
            debug!("Setting GPU mode: {}", GpuMode::Optimus);
        });

        let gfx_dbus = self.bios_proxy.clone();
        let states = self.states.clone();
        gpu_menu.add("Ultimate", move |_| {
            if let Ok(mut lock) = states.lock() {
                lock.tray_should_update = true;
            }
            gfx_dbus
                .set_gpu_mux_mode(GpuMode::Discrete)
                .map_err(|e| {
                    error!("ROGTray: set_mode: {e}");
                    e
                })
                .ok();
            debug!("Setting GPU mode: {}", GpuMode::Discrete);
        });

        let active = match current_mode {
            GfxMode::AsusMuxDgpu => "Ultimate".to_owned(),
            GfxMode::Hybrid => "Optimus".to_owned(),
            _ => current_mode.to_string(),
        };
        debug!("Current active GPU mode: {}", active);
        let reboot_required = if reboot_required {
            "(Reboot required)"
        } else {
            ""
        };
        self.add_radio_sub_menu(
            &format!("GPU Mode: {active} {reboot_required}"),
            active.as_str(),
            &gpu_menu,
        );

        debug!("ROGTray: appended gpu menu");
    }

    fn menu_clear(&mut self) {
        self.menu = gtk::Menu::new();
        debug!("ROGTray: cleared self");
    }

    /// Reset GTK menu to internal state, this can be called after clearing and
    /// rebuilding the menu too.
    fn menu_update(&mut self) {
        self.tray.set_menu(&mut self.menu);
        self.set_icon(self.icon);
    }

    /// Do a flush, build, and update of the tray menu
    fn rebuild_and_update(
        &mut self,
        supported_properties: &[Properties],
        supported_gfx: &[GfxMode],
        current_gfx_mode: GfxMode,
        charge_limit: Option<u8>,
        panel_od: Option<bool>,
        mini_led: Option<bool>,
    ) {
        self.menu_clear();
        self.menu_add_base();
        if let Some(charge_limit) = charge_limit {
            self.menu_add_charge_limit(charge_limit);
        }
        if let Some(panel_od) = panel_od {
            self.menu_add_panel_od(panel_od);
        }
        if let Some(mini_led) = mini_led {
            self.menu_add_mini_led_mode(mini_led);
        }
        if self.gfx_proxy_is_active {
            // Add a supergfxctl specific menu
            self.menu_add_supergfx(supported_gfx, current_gfx_mode);
        } else if supported_properties.contains(&Properties::GpuMuxMode) {
            self.menu_add_mux(current_gfx_mode);
        }
        self.menu_update();
    }
}

/// The tray is controlled somewhat by `Arc<Mutex<SystemState>>`
pub fn init_tray(supported_properties: Vec<Properties>, states: Arc<Mutex<SystemState>>) {
    std::thread::spawn(move || {
        let gtk_init = gtk::init().map_err(|e| {
            error!("ROGTray: gtk init {e}");
            e
        });
        if gtk_init.is_err() {
            return;
        } // Make this the main thread for gtk
        debug!("init_tray gtk");

        let mut tray = match ROGTray::new(states.clone()) {
            Ok(t) => {
                info!("init_tray: built menus");
                t
            }
            Err(e) => {
                error!("ROGTray: tray init {e}");
                if let Ok(mut states) = states.lock() {
                    states.error = Some(format!("Could not start tray: {e}"));
                }
                return;
            }
        };

        let supported_gfx = if tray.gfx_proxy_is_active {
            if let Ok(version) = tray.gfx_proxy.version() {
                if let Some(version) = Versioning::new(&version) {
                    let curr_gfx = Versioning::new("5.0.3-RC4").unwrap();
                    warn!("supergfxd version = {version}");
                    if version < curr_gfx {
                        // Don't allow mode changing if too old a version
                        warn!("supergfxd found but is too old to use");
                        tray.gfx_proxy_is_active = false;
                    }
                }
            }
            if tray.gfx_proxy_is_active {
                tray.gfx_proxy.supported().unwrap()
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        tray.rebuild_and_update(
            &supported_properties,
            &supported_gfx,
            GfxMode::Hybrid,
            None,
            None,
            None,
        );
        tray.set_icon(TRAY_APP_ICON);
        info!("Started ROGTray");

        loop {
            let states = tray.states.clone();
            if let Ok(mut lock) = states.lock() {
                if lock.tray_should_update {
                    // Supergfx ends up adding some complexity to handle if it isn't available
                    let current_gpu_mode = if lock.gfx_state.has_supergfx {
                        lock.gfx_state.mode
                    } else if let Some(mode) = lock.bios.gpu_mux_mode {
                        match mode {
                            GpuMode::Discrete => GfxMode::AsusMuxDgpu,
                            _ => GfxMode::Hybrid,
                        }
                    } else {
                        GfxMode::Hybrid
                    };
                    tray.rebuild_and_update(
                        &supported_properties,
                        &supported_gfx,
                        current_gpu_mode,
                        lock.bios.charge_limit,
                        lock.bios.panel_overdrive,
                        lock.bios.mini_led_mode,
                    );
                    lock.tray_should_update = false;
                    debug!("ROGTray: rebuilt menus due to state change");

                    match lock.gfx_state.power_status {
                        GfxPower::Suspended => tray.set_icon("asus_notif_blue"),
                        GfxPower::Off => {
                            if lock.gfx_state.mode == GfxMode::Vfio {
                                tray.set_icon("asus_notif_red")
                            } else {
                                tray.set_icon("asus_notif_green")
                            }
                        }
                        GfxPower::AsusDisabled => tray.set_icon("asus_notif_white"),
                        GfxPower::AsusMuxDiscreet | GfxPower::Active => {
                            tray.set_icon("asus_notif_red");
                        }
                        GfxPower::Unknown => {
                            if tray.gfx_proxy_is_active {
                                tray.set_icon("gpu-integrated");
                            } else {
                                tray.set_icon("asus_notif_red");
                            }
                        }
                    };
                }
            }

            if gtk::events_pending() {
                // This is blocking until any events are available
                gtk::main_iteration();
                continue;
            }
            // Don't spool at max speed if no gtk events
            std::thread::sleep(Duration::from_millis(50));
            trace!("Tray loop ticked");
        }
    });
}
