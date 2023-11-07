pub mod app;
use std::fs::{remove_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

pub use app::RogApp;

pub mod cli_options;
pub mod config;
pub mod error;
#[cfg(feature = "mocking")]
pub mod mocking;
pub mod pages;
pub mod startup_error;
pub mod system_state;
pub mod tray;
pub mod update_and_notify;
pub mod widgets;

#[cfg(feature = "mocking")]
pub use mocking::RogDbusClientBlocking;
use nix::sys::stat;
use nix::unistd;
#[cfg(not(feature = "mocking"))]
pub use rog_dbus::RogDbusClientBlocking;
use tempfile::TempDir;
// use log::{error, info, warn};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_versions() {
    println!("App and daemon versions:");
    println!("      rog-gui v{}", VERSION);
    println!("        asusd v{}", asusd::VERSION);
    println!("\nComponent crate versions:");
    println!("    rog-anime v{}", rog_anime::VERSION);
    println!("     rog-aura v{}", rog_aura::VERSION);
    println!("     rog-dbus v{}", rog_dbus::VERSION);
    println!(" rog-profiles v{}", rog_profiles::VERSION);
    println!("rog-platform v{}", rog_platform::VERSION);
}

pub const SHOWING_GUI: u8 = 1;
pub const SHOW_GUI: u8 = 2;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Page {
    AppSettings,
    System,
    AuraEffects,
    AnimeMatrix,
    FanCurves,
}

/// Either exit the process, or return with a refreshed tmp-dir
pub fn on_tmp_dir_exists() -> Result<TempDir, std::io::Error> {
    let mut buf = [0u8; 4];
    let path = std::env::temp_dir().join("rog-gui");

    if path.read_dir()?.next().is_none() {
        std::fs::remove_dir_all(path)?;
        return tempfile::Builder::new()
            .prefix("rog-gui")
            .rand_bytes(0)
            .tempdir();
    }

    let mut ipc_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(path.join("ipc.pipe"))?;

    // If the app is running this ends up stacked on top of SHOWING_GUI
    ipc_file.write_all(&[SHOW_GUI])?;
    // tiny sleep to give the app a chance to respond
    sleep(Duration::from_millis(100));
    ipc_file.read(&mut buf).ok();

    // First entry is the actual state
    if buf[0] == SHOWING_GUI {
        ipc_file.write_all(&[SHOWING_GUI])?; // Store state again as we drained the fifo
                                             // Early exit is not an error and we don't want to pass back a dir
        #[allow(clippy::exit)]
        exit(0);
    } else if buf[0] == SHOW_GUI {
        remove_dir_all(&path)?;
        return tempfile::Builder::new()
            .prefix("rog-gui")
            .rand_bytes(0)
            .tempdir();
    }
    panic!("Invalid exit or app state");
}

pub fn get_ipc_file() -> Result<File, crate::error::Error> {
    let tmp_dir = std::env::temp_dir().join("rog-gui");
    let fifo_path = tmp_dir.join("ipc.pipe");
    if let Err(e) = unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU) {
        if !matches!(e, nix::errno::Errno::EEXIST) {
            Err(e)?
        }
    }
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        // .truncate(true)
        .open(&fifo_path)?)
}
