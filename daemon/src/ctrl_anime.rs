const INIT_STR: &str = "ASUS Tech.Inc.";
const PACKET_SIZE: usize = 640;

// Only these two packets must be 17 bytes
const DEV_PAGE: u8 = 0x5e;
// These bytes are in [1] position of the array
const WRITE: u8 = 0xc0;
const INIT: u8 = 0xc2;
const SET: u8 = 0xc3;
const APPLY: u8 = 0xc4;

// Used to turn the panel on and off
// The next byte can be 0x03 for "on" and 0x00 for "off"
const ON_OFF: u8 = 0x04;

use rog_types::error::AuraError;
use log::{error, info, warn};
use rusb::{Device, DeviceHandle};
use std::convert::TryInto;
use std::error::Error;
use std::time::Duration;
use zbus::dbus_interface;

use crate::GetSupported;

use serde_derive::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct AnimeSupportedFunctions(bool);

impl GetSupported for CtrlAnimeDisplay {
    type A = AnimeSupportedFunctions;

    fn get_supported() -> Self::A {
        AnimeSupportedFunctions(CtrlAnimeDisplay::get_device(0x0b05, 0x193b).is_ok())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum AnimatrixCommand {
    Apply,
    SetBoot(bool),
    Write(Vec<u8>),
    WriteImage(Vec<Vec<u8>>),
    //ReloadLast,
}

pub struct CtrlAnimeDisplay {
    handle: DeviceHandle<rusb::GlobalContext>,
    initialised: bool,
}

//AnimatrixWrite
pub trait Dbus {
    fn set_anime(&mut self, input: Vec<Vec<u8>>);

    fn set_on_off(&mut self, status: bool);

    fn set_boot_on_off(&mut self, status: bool);
}

impl crate::ZbusAdd for CtrlAnimeDisplay {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(&"/org/asuslinux/Anime".try_into().unwrap(), self)
            .map_err(|err| {
                warn!("CtrlAnimeDisplay: add_to_server {}", err);
                err
            })
            .ok();
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl Dbus for CtrlAnimeDisplay {
    fn set_anime(&mut self, input: Vec<Vec<u8>>) {
        self.do_command(AnimatrixCommand::WriteImage(input))
            .map_or_else(|err| warn!("{}", err), |()| info!("Writing image to Anime"));
    }

    fn set_on_off(&mut self, status: bool) {
        let mut flush: Vec<u8> = vec![0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = WRITE;
        flush[2] = ON_OFF;

        let status_str;
        if status {
            flush[3] = 0x03;
            status_str = "on";
        } else {
            flush[3] = 0x00;
            status_str = "off";
        }

        self.do_command(AnimatrixCommand::Write(flush)).map_or_else(
            |err| warn!("{}", err),
            |()| info!("Turning {} the AniMe", status_str),
        );
    }

    fn set_boot_on_off(&mut self, status: bool) {
        let status_str = if status { "on" } else { "off" };

        self.do_command(AnimatrixCommand::SetBoot(status))
            .and_then(|()| self.do_command(AnimatrixCommand::Apply))
            .map_or_else(
                |err| warn!("{}", err),
                |()| info!("Turning {} the AniMe at boot/shutdown", status_str),
            );
    }
}

impl CtrlAnimeDisplay {
    #[inline]
    pub fn new() -> Result<CtrlAnimeDisplay, Box<dyn Error>> {
        // We don't expect this ID to ever change
        let device = CtrlAnimeDisplay::get_device(0x0b05, 0x193b)?;

        let mut device = device.open()?;
        device.reset()?;

        device.set_auto_detach_kernel_driver(true).map_err(|err| {
            error!("Auto-detach kernel driver failed: {}", err);
            err
        })?;

        device.claim_interface(0).map_err(|err| {
            error!("Could not claim device interface: {}", err);
            err
        })?;

        info!("Device has an AniMe Matrix display");
        Ok(CtrlAnimeDisplay {
            handle: device,
            initialised: false,
        })
    }

    #[inline]
    fn get_device(vendor: u16, product: u16) -> Result<Device<rusb::GlobalContext>, rusb::Error> {
        for device in rusb::devices()?.iter() {
            let device_desc = device.device_descriptor()?;
            if device_desc.vendor_id() == vendor && device_desc.product_id() == product {
                return Ok(device);
            }
        }
        Err(rusb::Error::NoDevice)
    }

    pub fn do_command(&mut self, command: AnimatrixCommand) -> Result<(), AuraError> {
        if !self.initialised {
            self.do_initialization()?
        }

        match command {
            AnimatrixCommand::Apply => self.do_apply()?,
            //AnimatrixCommand::Set => self.do_set_boot()?,
            AnimatrixCommand::SetBoot(status) => self.do_set_boot(status)?,
            AnimatrixCommand::Write(bytes) => self.write_bytes(&bytes)?,
            AnimatrixCommand::WriteImage(effect) => self.write_image(effect)?,
            //AnimatrixCommand::ReloadLast => self.reload_last_builtin(&config).await?,
        }
        Ok(())
    }

    /// Should only be used if the bytes you are writing are verified correct
    #[inline]
    fn write_bytes(&self, message: &[u8]) -> Result<(), AuraError> {
        match self.handle.write_control(
            0x21,  // request_type
            0x09,  // request
            0x35e, // value
            0x00,  // index
            message,
            Duration::from_millis(200),
        ) {
            Ok(_) => {}
            Err(err) => match err {
                rusb::Error::Timeout => {}
                _ => error!("Failed to write to led interrupt: {}", err),
            },
        }
        Ok(())
    }

    /// Write an Animatrix image
    ///
    /// The expected input here is *two* Vectors, 640 bytes in length. The two vectors
    /// are each one half of the full image write.
    ///
    /// After each write a flush is written, it is assumed that this tells the device to
    /// go ahead and display the written bytes
    ///
    /// # Note:
    /// The vectors are expected to contain the full sequence of bytes as follows
    ///
    /// - Write pane 1: 0x5e 0xc0 0x02 0x01 0x00 0x73 0x02 .. <led brightness>
    /// - Write pane 2: 0x5e 0xc0 0x02 0x74 0x02 0x73 0x02 .. <led brightness>
    ///
    /// Where led brightness is 0..255, low to high
    #[inline]
    fn write_image(&mut self, image: Vec<Vec<u8>>) -> Result<(), AuraError> {
        for row in image.iter() {
            self.write_bytes(row)?;
        }
        self.do_flush()?;
        Ok(())
    }

    #[inline]
    fn do_initialization(&mut self) -> Result<(), AuraError> {
        let mut init = [0; PACKET_SIZE];
        init[0] = DEV_PAGE; // This is the USB page we're using throughout
        for (idx, byte) in INIT_STR.as_bytes().iter().enumerate() {
            init[idx + 1] = *byte
        }
        self.write_bytes(&init)?;

        // clear the init array and write other init message
        for ch in init.iter_mut() {
            *ch = 0;
        }
        init[0] = DEV_PAGE; // write it to be sure?
        init[1] = INIT;

        self.write_bytes(&init)?;
        self.initialised = true;
        Ok(())
    }

    #[inline]
    fn do_flush(&mut self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = WRITE;
        flush[2] = 0x03;

        self.write_bytes(&flush)?;
        Ok(())
    }

    #[inline]
    fn do_set_boot(&mut self, status: bool) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = SET;
        flush[2] = 0x01;
        flush[3] = if status { 0x00 } else { 0x80 };

        self.write_bytes(&flush)?;
        Ok(())
    }

    #[inline]
    fn do_apply(&mut self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = APPLY;
        flush[2] = 0x01;
        flush[3] = 0x80;

        self.write_bytes(&flush)?;
        Ok(())
    }
}
