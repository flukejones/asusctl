//! A seld-contained tray icon with menus. The control of app<->tray is done via
//! commands over an MPSC channel.

use std::{
    io::Write,
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
    time::Duration,
};

use gtk::{gio::Icon, prelude::*};
use rog_dbus::zbus_platform::RogBiosProxyBlocking;
use rog_platform::{platform::GpuMode, supported::SupportedFunctions};

use crate::{error::Result, get_ipc_file, page_states::PageDataStates, SHOW_GUI};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use supergfxctl::{
    pci_device::{GfxMode, GfxPower},
    zbus_proxy::DaemonProxyBlocking as GfxProxyBlocking,
};

use log::trace;

const TRAY_APP_ICON: &str = "rog-control-center";
const TRAY_LABEL: &str = "ROG Control Center";

pub enum AppToTray {
    DgpuStatus(GfxPower),
}

pub enum TrayToApp {
    Open,
    Quit,
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
    bios_proxy: RogBiosProxyBlocking<'static>,
    gfx_proxy: GfxProxyBlocking<'static>,
}

impl ROGTray {
    pub fn new() -> Result<Self> {
        let conn = zbus::blocking::Connection::system().unwrap();
        let rog_tray = Self {
            tray: AppIndicator::new(TRAY_LABEL, TRAY_APP_ICON),
            menu: gtk::Menu::new(),
            icon: TRAY_APP_ICON,
            bios_proxy: RogBiosProxyBlocking::new(&conn).unwrap(),
            gfx_proxy: GfxProxyBlocking::new(&conn).unwrap(),
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

    fn add_radio_sub_menu(&mut self, header_label: &str, active_label: &str, sub_menu: RadioGroup) {
        let header_item = gtk::MenuItem::with_label(header_label);
        header_item.show_all();
        self.menu.add(&header_item);

        let menu = gtk::Menu::new();
        for item in sub_menu.0.iter() {
            item.set_active(item.label().unwrap() == active_label);
            item.set_no_show_all(false);
            item.show_all();
            menu.add(item);
        }
        menu.show_all();
        header_item.set_submenu(Some(&menu));
    }

    fn add_menu_item<F>(&mut self, label: &str, cb: F)
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

    fn set_status(&mut self, status: AppIndicatorStatus) {
        self.tray.set_status(status)
    }

    fn menu_add_base(&mut self) {
        self.add_icon_menu_item("Open app", "asus_notif_red", move || {
            get_ipc_file().unwrap().write_all(&[SHOW_GUI]).ok();
        });

        self.add_separator();
    }

    fn menu_add_charge_limit(&mut self, limit: u8) {
        self.add_inactive_label(&format!("Charge limit: {limit}"));
    }

    fn menu_add_panel_od(&mut self, panel_od: bool) {
        let bios = self.bios_proxy.clone();
        self.add_check_menu_item("Panel Overdrive", panel_od, move |this| {
            bios.set_panel_od(this.is_active()).unwrap();
        });
    }

    fn menu_add_gpu(&mut self, supported: &SupportedFunctions, current_mode: GfxMode) {
        let gfx_dbus = self.gfx_proxy.clone();
        let mut gpu_menu = RadioGroup::new("Integrated", move |_| {
            let mode = gfx_dbus.mode().unwrap();
            if mode != GfxMode::Integrated {
                gfx_dbus.set_mode(&GfxMode::Integrated).unwrap();
            }
        });

        let gfx_dbus = self.gfx_proxy.clone();
        gpu_menu.add("Hybrid", move |_| {
            let mode = gfx_dbus.mode().unwrap();
            if mode != GfxMode::Hybrid {
                gfx_dbus.set_mode(&GfxMode::Hybrid).unwrap();
            }
        });
        if supported.rog_bios_ctrl.gpu_mux {
            let gfx_dbus = self.bios_proxy.clone();
            gpu_menu.add("Ultimate (Reboot required)", move |_| {
                let mode = gfx_dbus.gpu_mux_mode().unwrap();
                if mode != GpuMode::Discrete {
                    gfx_dbus.set_gpu_mux_mode(GpuMode::Discrete).unwrap();
                }
            });
        }
        if supported.rog_bios_ctrl.egpu_enable {
            let gfx_dbus = self.gfx_proxy.clone();
            gpu_menu.add("eGPU", move |_| {
                let mode = gfx_dbus.mode().unwrap();
                if mode != GfxMode::Egpu {
                    gfx_dbus.set_mode(&GfxMode::Egpu).unwrap();
                }
            });
        }

        let active = match current_mode {
            GfxMode::AsusMuxDiscreet => "Discreet".to_string(),
            _ => current_mode.to_string(),
        };
        self.add_radio_sub_menu(
            &format!("GPU Mode: {current_mode}"),
            active.as_str(),
            gpu_menu,
        );
    }

    fn menu_clear(&mut self) {
        self.menu = gtk::Menu::new();
    }

    /// Reset GTK menu to internal state, this can be called after clearing and rebuilding the menu too.
    fn menu_update(&mut self) {
        self.tray.set_menu(&mut self.menu);
        self.set_icon(self.icon);
    }

    /// Do a flush, build, and update of the tray menu
    fn rebuild_and_update(
        &mut self,
        supported: &SupportedFunctions,
        current_gfx_mode: GfxMode,
        charge_limit: u8,
        panel_od: bool,
    ) {
        self.menu_clear();
        self.menu_add_base();
        self.menu_add_charge_limit(charge_limit);
        self.menu_add_panel_od(panel_od);
        self.menu_add_gpu(supported, current_gfx_mode);
        self.menu_update();
    }
}

pub fn init_tray(
    supported: SupportedFunctions,
    states: Arc<Mutex<PageDataStates>>,
) -> Receiver<TrayToApp> {
    let (send, recv) = channel();
    let _send = Arc::new(Mutex::new(send));

    std::thread::spawn(move || {
        gtk::init().unwrap(); // Make this the main thread for gtk

        let mut tray = ROGTray::new().unwrap();
        tray.rebuild_and_update(&supported, GfxMode::Hybrid, 100, false);
        tray.set_icon(TRAY_APP_ICON);

        loop {
            if let Ok(mut lock) = states.lock() {
                if lock.tray_should_update {
                    tray.rebuild_and_update(
                        &supported,
                        lock.gfx_state.mode,
                        lock.power_state.charge_limit,
                        lock.bios.panel_overdrive,
                    );
                    lock.tray_should_update = false;

                    match lock.gfx_state.power_status {
                        GfxPower::Active => tray.set_icon("asus_notif_red"),
                        GfxPower::Suspended => tray.set_icon("asus_notif_blue"),
                        GfxPower::Off => tray.set_icon("asus_notif_green"),
                        GfxPower::AsusDisabled => tray.set_icon("asus_notif_white"),
                        GfxPower::AsusMuxDiscreet => tray.set_icon("asus_notif_red"),
                        GfxPower::Unknown => tray.set_icon("gpu-integrated"),
                    };
                }
            }

            if gtk::events_pending() {
                // This is blocking until any events are available
                gtk::main_iteration();
                continue;
            }
            // Don't spool at max speed if no gtk events
            std::thread::sleep(Duration::from_millis(300));
            trace!("Tray loop ticked");
        }
    });

    recv
}
