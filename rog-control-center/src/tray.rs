//! A seld-contained tray icon with menus. The control of app<->tray is done via
//! commands over an MPSC channel.

use std::{
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
    time::Duration,
    io::Write,
};

use supergfxctl::pci_device::GfxPower;
use tray_item::TrayItem;
use crate::{SHOW_GUI, get_ipc_file};

pub enum AppToTray {
    DgpuStatus(GfxPower),
}

pub enum TrayToApp {
    Open,
    Quit,
}

pub fn init_tray(recv_command: Receiver<AppToTray>) -> Receiver<TrayToApp> {
    let (send, recv) = channel();
    let send = Arc::new(Mutex::new(send));

    std::thread::spawn(move || {
        gtk::init().unwrap(); // Make this the main thread for gtk

        let mut tray = TrayItem::new("ROG Control Center", "rog-control-center").unwrap();
        
        tray.add_menu_item("Open app", move || {
            get_ipc_file().unwrap().write_all(&[SHOW_GUI]).ok();
        })
        .ok();
        
        let s1 = send.clone();
        tray.add_menu_item("Quit", move || {
            let lock = s1.lock().unwrap();
            lock.send(TrayToApp::Quit).ok();
        })
        .ok();

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
                        }
                        .ok();
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
