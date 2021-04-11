use log::{error, info, warn};
use rog_anime::{
    usb::{
        pkt_for_apply, pkt_for_flush, pkt_for_set_boot, pkt_for_set_on, pkts_for_init, PROD_ID,
        VENDOR_ID,
    },
    AnimeDataBuffer, AnimePacketType,
};
use rog_types::supported::AnimeSupportedFunctions;
use rusb::{Device, DeviceHandle};
use std::error::Error;
use std::time::Duration;
use zbus::dbus_interface;
use zvariant::ObjectPath;

use crate::GetSupported;

impl GetSupported for CtrlAnimeDisplay {
    type A = AnimeSupportedFunctions;

    fn get_supported() -> Self::A {
        AnimeSupportedFunctions(CtrlAnimeDisplay::get_device(VENDOR_ID, PROD_ID).is_ok())
    }
}

pub struct CtrlAnimeDisplay {
    handle: DeviceHandle<rusb::GlobalContext>,
}

impl crate::ZbusAdd for CtrlAnimeDisplay {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(
                &ObjectPath::from_str_unchecked("/org/asuslinux/Anime"),
                self,
            )
            .map_err(|err| {
                warn!("CtrlAnimeDisplay: add_to_server {}", err);
                err
            })
            .ok();
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlAnimeDisplay {
    /// Writes a data stream of length
    fn write(&self, input: AnimeDataBuffer) {
        self.write_data_buffer(input);
    }

    fn set_on_off(&self, status: bool) {
        self.write_bytes(&pkt_for_set_on(status));
    }

    fn set_boot_on_off(&self, on: bool) {
        self.write_bytes(&pkt_for_set_boot(on));
        self.write_bytes(&pkt_for_apply());
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

    fn get_device(vendor: u16, product: u16) -> Result<Device<rusb::GlobalContext>, rusb::Error> {
        for device in rusb::devices()?.iter() {
            let device_desc = device.device_descriptor()?;
            if device_desc.vendor_id() == vendor && device_desc.product_id() == product {
                return Ok(device);
            }
        }
        Err(rusb::Error::NoDevice)
    }

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

    fn write_data_buffer(&self, buffer: AnimeDataBuffer) {
        let data = AnimePacketType::from(buffer);
        for row in data.iter() {
            self.write_bytes(row);
        }
        self.write_bytes(&pkt_for_flush());
    }

    fn do_initialization(&self) {
        let pkts = pkts_for_init();
        self.write_bytes(&pkts[0]);
        self.write_bytes(&pkts[1]);
    }
}
