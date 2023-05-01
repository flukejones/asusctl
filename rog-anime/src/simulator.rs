use std::error::Error;

use log::error;
use rog_anime::usb::{PROD_ID, VENDOR_ID};
use uhid_virt::{Bus, CreateParams, UHIDDevice};

pub struct VirtAnimeMatrix {
    device: UHIDDevice<std::fs::File>,
}

impl Default for VirtAnimeMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtAnimeMatrix {
    pub fn new() -> Self {
        VirtAnimeMatrix {
            device: UHIDDevice::create(CreateParams {
                name: String::from("ROG_Virtual Anime Matrix"),
                phys: String::from(""),
                uniq: String::from(""),
                bus: Bus::USB,
                vendor: VENDOR_ID as u32,
                product: PROD_ID as u32,
                version: 0,
                country: 0,
                // This is a device which emits the usage code as a whole, rather than as bits
                rd_data: [
                    0x06, 0x31, 0xff, // Usage Page (Vendor Defined 0xFF31)
                    0x09, 0x76, // Usage (0x76)
                    0xa1, 0x01, // Collection (Application)
                    0x85, 0x5a, //   Report ID (90)
                    0x19, 0x00, //   Usage Minimum (0x00)
                    0x2a, 0xff, 0x00, //   Usage Maximum (0xFF)
                    0x15, 0x00, //   Logical Minimum (0)
                    0x26, 0xff, 0x00, //   Logical Maximum (255)
                    0x75, 0x08, //   Report Size (8)
                    0x95, 0x05, //   Report Count (5)
                    0x81,
                    0x00, /*   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null
                           * Position) */
                    0x19, 0x00, //   Usage Minimum (0x00)
                    0x2a, 0xff, 0x00, //   Usage Maximum (0xFF)
                    0x15, 0x00, //   Logical Minimum (0)
                    0x26, 0xff, 0x00, //   Logical Maximum (255)
                    0x75, 0x08, //   Report Size (8)
                    0x95, 0x3f, //   Report Count (63)
                    0xb1,
                    0x00, /*   Feature (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null
                           * Position,Non-volatile) */
                    0xc0, // End Collection
                    0x06, 0x31, 0xff, // Usage Page (Vendor Defined 0xFF31)
                    0x09, 0x80, // Usage (0x80)
                    0xa1, 0x01, // Collection (Application)
                    0x85, 0x5e, //   Report ID (94)
                    0x19, 0x00, //   Usage Minimum (0x00)
                    0x2a, 0xff, 0x00, //   Usage Maximum (0xFF)
                    0x15, 0x00, //   Logical Minimum (0)
                    0x26, 0xff, 0x00, //   Logical Maximum (255)
                    0x75, 0x08, //   Report Size (8)
                    0x95, 0x05, //   Report Count (5)
                    0x81,
                    0x00, /*   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null
                           * Position) */
                    0x19, 0x00, //   Usage Minimum (0x00)
                    0x2a, 0xff, 0x00, //   Usage Maximum (0xFF)
                    0x15, 0x00, //   Logical Minimum (0)
                    0x26, 0xff, 0x00, //   Logical Maximum (255)
                    0x75, 0x08, //   Report Size (8)
                    0x96, 0x7f, 0x02, //   Report Count (639)
                    0xb1,
                    0x00, /*   Feature (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null
                           * Position,Non-volatile) */
                    0xc0, /* End Collection */

                          /* 85 bytes */
                ]
                .to_vec(),
            })
            .map_err(|err| error!("Could not create virtual device: {:?}", err))
            .expect("Could not create virtual device"),
        }
    }

    /// A single on/off key press
    pub fn press(&mut self, input: [u8; 32]) {
        self.device.write(&input).unwrap();
        let mut reset = [0u8; 32];
        reset[0] = input[0];
        self.device.write(&reset).unwrap();
    }

    pub fn read(&mut self) {
        if let Ok(event) = self.device.read() {
            match event {
                // uhid_virt::OutputEvent::Start { dev_flags } => todo!(),
                // uhid_virt::OutputEvent::Stop => todo!(),
                // uhid_virt::OutputEvent::Open => todo!(),
                // uhid_virt::OutputEvent::Close => todo!(),
                uhid_virt::OutputEvent::Output { data } => {
                    dbg!(data);
                }
                uhid_virt::OutputEvent::GetReport {
                    id,
                    report_number,
                    report_type,
                } => {
                    dbg!(id, report_number, report_type);
                }
                // uhid_virt::OutputEvent::SetReport { id, report_number, report_type, data } =>
                // todo!(),
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut dev = VirtAnimeMatrix::new();

    loop {
        dev.read();
    }
}
