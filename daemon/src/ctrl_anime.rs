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
use rog_types::{
    anime_matrix::{
        AniMeDataBuffer, AniMeImageBuffer, AniMePacketType, ANIME_PANE1_PREFIX, ANIME_PANE2_PREFIX,
    },
    error::AuraError,
};
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

pub struct CtrlAnimeDisplay {
    handle: DeviceHandle<rusb::GlobalContext>,
}

//AnimatrixWrite
pub trait Dbus {
    /// Write an image 34x56 pixels. Each pixel is 0-255 greyscale.
    fn write_image(&self, input: AniMeImageBuffer);

    /// Write a direct stream of data
    fn write_direct(&self, input: AniMeDataBuffer);

    fn set_on_off(&self, status: bool);

    fn set_boot_on_off(&self, status: bool);
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
    /// Writes a 34x56 image
    fn write_image(&self, input: AniMeImageBuffer) {
        self.write_image_buffer(input)
            .map_or_else(|err| warn!("{}", err), |()| info!("Writing image to Anime"));
    }

    /// Writes a data stream of length
    fn write_direct(&self, input: AniMeDataBuffer) {
        self.write_data_buffer(input)
            .map_or_else(|err| warn!("{}", err), |()| info!("Writing data to Anime"));
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
        let status_str = if status { "on" } else { "off" };

        self.do_set_boot(status).map_or_else(
            |err| warn!("{}", err),
            |()| info!("Turning {} the AniMe at boot/shutdown", status_str),
        );
        self.do_apply().map_or_else(
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
        let ctrl = CtrlAnimeDisplay {
            handle: device,
        };
        ctrl.do_initialization()?;

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
    fn write_data_buffer(&self, buffer: AniMeDataBuffer) -> Result<(), AuraError> {
        let mut image = AniMePacketType::from(buffer);
        image[0][..7].copy_from_slice(&ANIME_PANE1_PREFIX);
        image[1][..7].copy_from_slice(&ANIME_PANE2_PREFIX);

        for row in image.iter() {
            self.write_bytes(row);
        }
        self.do_flush()?;
        Ok(())
    }

    /// Write an Animatrix image
    ///
    /// The expected USB input here is *two* Vectors, 640 bytes in length. The two vectors
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
    fn write_image_buffer(&self, buffer: AniMeImageBuffer) -> Result<(), AuraError> {
        let mut image = AniMePacketType::from(buffer);
        image[0][..7].copy_from_slice(&ANIME_PANE1_PREFIX);
        image[1][..7].copy_from_slice(&ANIME_PANE2_PREFIX);

        for row in image.iter() {
            self.write_bytes(row);
        }
        self.do_flush()?;
        Ok(())
    }

    #[inline]
    fn do_initialization(&self) -> Result<(), AuraError> {
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
        Ok(())
    }

    #[inline]
    fn do_flush(&self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = WRITE;
        flush[2] = 0x03;

        self.write_bytes(&flush);
        Ok(())
    }

    #[inline]
    fn do_set_boot(&self, status: bool) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = SET;
        flush[2] = 0x01;
        flush[3] = if status { 0x00 } else { 0x80 };

        self.write_bytes(&flush);
        Ok(())
    }

    #[inline]
    fn do_apply(&self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = APPLY;
        flush[2] = 0x01;
        flush[3] = 0x80;

        self.write_bytes(&flush);
        Ok(())
    }
}
