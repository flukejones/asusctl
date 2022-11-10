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

use gtk::{gio::Icon, prelude::*, MenuItem};
use rog_platform::{platform::GpuMode, supported::SupportedFunctions};

use crate::{error::Result, get_ipc_file, SHOW_GUI};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use supergfxctl::pci_device::{GfxMode, GfxPower};

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
    items: Vec<MenuItem>,
}

impl ROGTray {
    pub fn new() -> Result<Self> {
        let mut rog_tray = Self {
            tray: AppIndicator::new(TRAY_LABEL, TRAY_APP_ICON),
            menu: gtk::Menu::new(),
            items: Vec::new(),
        };
        // rog_tray.set_icon(TRAY_APP_ICON);

        rog_tray.add_icon_menu_item("Open app", "asus_notif_red", move || {
            get_ipc_file().unwrap().write_all(&[SHOW_GUI]).ok();
        });

        rog_tray.add_inactive_label("----");

        Ok(rog_tray)
    }

    pub fn set_icon(&mut self, icon: &str) {
        self.tray.set_icon(icon);
        self.tray.set_status(AppIndicatorStatus::Active);
    }

    /// Add a non-interactive label
    fn add_inactive_label(&mut self, label: &str) {
        let item = gtk::MenuItem::with_label(label);
        item.set_sensitive(false);
        self.menu.append(&item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);
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
        self.items.push(item);
        let item = self.items.last().unwrap();
        item.connect_activate(move |_| {
            cb();
        });
        self.menu.append(item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);
    }

    /// Add a menu item with an icon to the right
    fn add_icon_menu_item<F>(&mut self, label: &str, icon: &str, cb: F)
    where
        F: Fn() + Send + 'static,
    {
        let g_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let icon = gtk::Image::from_gicon(&Icon::for_string(icon).unwrap(), gtk::IconSize::Menu);
        let label = gtk::Label::new(Some(label));
        let menu_item = gtk::MenuItem::new();
        g_box.add(&icon);
        g_box.add(&label);

        menu_item.add(&g_box);
        menu_item.show_all();

        self.items.push(menu_item);
        let item = self.items.last().unwrap();
        item.connect_activate(move |_| {
            cb();
        });
        self.menu.append(item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);
    }

    fn set_status(&mut self, status: AppIndicatorStatus) {
        self.tray.set_status(status)
    }
}

fn init_gpu_menu(supported: &SupportedFunctions, tray: &mut ROGTray) {
    let conn = zbus::blocking::Connection::system().unwrap();
    let gfx_dbus = supergfxctl::zbus_proxy::DaemonProxyBlocking::new(&conn).unwrap();

    let mode = gfx_dbus.mode().unwrap();
    let mut gpu_menu = RadioGroup::new("Integrated", move |_| {
        let mode = gfx_dbus.mode().unwrap();
        if mode != GfxMode::Integrated {
            gfx_dbus.set_mode(&GfxMode::Integrated).unwrap();
        }
    });
    let gfx_dbus = supergfxctl::zbus_proxy::DaemonProxyBlocking::new(&conn).unwrap();
    gpu_menu.add("Hybrid", move |_| {
        let mode = gfx_dbus.mode().unwrap();
        if mode != GfxMode::Hybrid {
            gfx_dbus.set_mode(&GfxMode::Hybrid).unwrap();
        }
    });
    if supported.rog_bios_ctrl.gpu_mux {
        let gfx_dbus = rog_dbus::zbus_platform::RogBiosProxyBlocking::new(&conn).unwrap();
        gpu_menu.add("Ultimate (Reboot required)", move |_| {
            let mode = gfx_dbus.gpu_mux_mode().unwrap();
            if mode != GpuMode::Discrete {
                gfx_dbus.set_gpu_mux_mode(GpuMode::Discrete).unwrap();
            }
        });
    }
    if supported.rog_bios_ctrl.egpu_enable {
        let gfx_dbus = supergfxctl::zbus_proxy::DaemonProxyBlocking::new(&conn).unwrap();
        gpu_menu.add("eGPU", move |_| {
            let mode = gfx_dbus.mode().unwrap();
            if mode != GfxMode::Egpu {
                gfx_dbus.set_mode(&GfxMode::Egpu).unwrap();
            }
        });
    }

    let active = match mode {
        GfxMode::AsusMuxDiscreet => "Discreet".to_string(),
        _ => mode.to_string(),
    };
    tray.add_radio_sub_menu("GPU Mode", active.as_str(), gpu_menu);
}

pub fn init_tray(
    supported: SupportedFunctions,
    recv_command: Receiver<AppToTray>,
) -> Receiver<TrayToApp> {
    let (send, recv) = channel();
    let _send = Arc::new(Mutex::new(send));

    std::thread::spawn(move || {
        gtk::init().unwrap(); // Make this the main thread for gtk

        let mut tray = ROGTray::new().unwrap();

        init_gpu_menu(&supported, &mut tray);
        // let s1 = send.clone();
        // tray.add_menu_item("Quit", move || {
        //     let lock = s1.lock().unwrap();
        //     lock.send(TrayToApp::Quit).ok();
        // })
        // .ok();

        //***********************************
        // finally do
        tray.tray.set_menu(&mut tray.menu);
        //***********************************

        loop {
            if let Ok(command) = recv_command.try_recv() {
                match command {
                    AppToTray::DgpuStatus(s) => {
                        match s {
                            GfxPower::Active => tray.set_icon("asus_notif_red"),
                            GfxPower::Suspended => tray.set_icon("asus_notif_blue"),
                            GfxPower::Off => tray.set_icon("asus_notif_green"),
                            GfxPower::AsusDisabled => tray.set_icon("asus_notif_white"),
                            GfxPower::AsusMuxDiscreet => tray.set_icon("asus_notif_red"),
                            GfxPower::Unknown => tray.set_icon("gpu-integrated"),
                        };
                    }
                }
            }

            if gtk::events_pending() {
                // This is blocking until any events are available
                gtk::main_iteration();
            } else {
                // Don't spool at max speed if no gtk events
                std::thread::sleep(Duration::from_millis(300));
            }
        }
    });

    recv
}
