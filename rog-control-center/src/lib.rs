pub mod app;
use std::{
    fs::{remove_dir_all, File, OpenOptions},
    io::{Read, Write},
    process::exit,
    thread::sleep,
    time::Duration,
};

pub use app::RogApp;

pub mod config;
pub mod error;
#[cfg(feature = "mocking")]
pub mod mocking;
pub mod notify;
pub mod page_states;
pub mod pages;
pub mod startup_error;
pub mod widgets;

#[cfg(feature = "mocking")]
pub use mocking::RogDbusClientBlocking;
#[cfg(not(feature = "mocking"))]
pub use rog_dbus::RogDbusClientBlocking;

use nix::{sys::stat, unistd};
use tempfile::TempDir;
//use log::{error, info, warn};

pub const SHOWING_GUI: u8 = 1;
pub const SHOW_GUI: u8 = 2;

#[derive(PartialEq, Clone, Copy)]
pub enum Page {
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
    sleep(Duration::from_millis(10));
    ipc_file.read(&mut buf).ok();

    // First entry is the actual state
    if buf[0] == SHOWING_GUI {
        ipc_file.write_all(&[SHOWING_GUI])?; // Store state again as we drained the fifo
        exit(0);
    } else if buf[0] == SHOW_GUI {
        remove_dir_all(&path)?;
        return tempfile::Builder::new()
            .prefix("rog-gui")
            .rand_bytes(0)
            .tempdir();
    }
    exit(-1);
}

pub fn get_ipc_file() -> Result<File, crate::error::Error> {
    let tmp_dir = std::env::temp_dir().join("rog-gui");
    let fifo_path = tmp_dir.join("ipc.pipe");
    if let Err(e) = unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU) {
        if !matches!(e, nix::Error::Sys(nix::errno::Errno::EEXIST)) {
            return Err(e)?;
        }
    }
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(&fifo_path)?)
}
