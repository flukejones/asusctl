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

use log::{error, info, warn};
use rog_anime::{AniMeDataBuffer, AniMePacketType};
use rusb::{Device, DeviceHandle};
use zvariant::ObjectPath;
use std::error::Error;
use std::time::Duration;
use zbus::dbus_interface;

use crate::GetSupported;

use serde_derive::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct AnimeSupportedFunctions(pub bool);

impl GetSupported for CtrlAnimeDisplay {
    type A = AnimeSupportedFunctions;

    fn get_supported() -> Self::A {
        AnimeSupportedFunctions(CtrlAnimeDisplay::get_device(0x0b05, 0x193b).is_ok())
    }
}

pub struct CtrlAnimeDisplay {
    handle: DeviceHandle<rusb::GlobalContext>,
}

//AnimatrixWrite
pub trait Dbus {
    /// Write a direct stream of data
    fn write(&self, input: AniMeDataBuffer);

    fn set_on_off(&self, status: bool);

    fn set_boot_on_off(&self, status: bool);
}

impl crate::ZbusAdd for CtrlAnimeDisplay {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(&ObjectPath::from_str_unchecked("/org/asuslinux/Anime"), self)
            .map_err(|err| {
                warn!("CtrlAnimeDisplay: add_to_server {}", err);
                err
            })
            .ok();
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl Dbus for CtrlAnimeDisplay {
    /// Writes a data stream of length
    fn write(&self, input: AniMeDataBuffer) {
        self.write_data_buffer(input);
    }

    fn set_on_off(&self, status: bool) {
        let mut buffer = [0u8; PACKET_SIZE];
        buffer[0] = DEV_PAGE;
        buffer[1] = WRITE;
        buffer[2] = ON_OFF;

        if status {
            buffer[3] = 0x03;
        } else {
            buffer[3] = 0x00;
        }

        self.write_bytes(&buffer);
    }

    fn set_boot_on_off(&self, status: bool) {
        self.do_set_boot(status);
        self.do_apply();
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
        let ctrl = CtrlAnimeDisplay { handle: device };
        ctrl.do_initialization();

        Ok(ctrl)
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

    /// Should only be used if the bytes you are writing are verified correct
    #[inline]
    fn write_bytes(&self, message: &[u8]) {
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
    }
    #[inline]
    fn write_data_buffer(&self, buffer: AniMeDataBuffer) {
        let data = AniMePacketType::from(buffer);
        for row in data.iter() {
            self.write_bytes(row);
        }
        self.do_flush();
    }

    #[inline]
    fn do_initialization(&self) {
        let mut init = [0; PACKET_SIZE];
        init[0] = DEV_PAGE; // This is the USB page we're using throughout
        for (idx, byte) in INIT_STR.as_bytes().iter().enumerate() {
            init[idx + 1] = *byte
        }
        self.write_bytes(&init);

        // clear the init array and write other init message
        for ch in init.iter_mut() {
            *ch = 0;
        }
        init[0] = DEV_PAGE; // write it to be sure?
        init[1] = INIT;

        self.write_bytes(&init);
    }

    #[inline]
    fn do_flush(&self) {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = WRITE;
        flush[2] = 0x03;

        self.write_bytes(&flush);
    }

    #[inline]
    fn do_set_boot(&self, status: bool) {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = SET;
        flush[2] = 0x01;
        flush[3] = if status { 0x00 } else { 0x80 };

        self.write_bytes(&flush);
    }

    #[inline]
    fn do_apply(&self) {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = APPLY;
        flush[2] = 0x01;
        flush[3] = 0x80;

        self.write_bytes(&flush);
    }
}
