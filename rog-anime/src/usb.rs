//! Utils for writing to the AniMe USB device
//!
//! Use of the device requires a few steps:
//! 1. Initialise the device by writing the two packets from `get_init_packets()`
//! 2. Write data from `AnimePacketType`
//! 3. Write the packet from `get_flush_packet()`, which tells the device to display the data from step 2
//!
//! Step 1 need to applied only on fresh system boot.

use crate::{error::AnimeError, AnimeType};

const INIT_STR: [u8; 15] = [
    0x5e, b'A', b'S', b'U', b'S', b' ', b'T', b'e', b'c', b'h', b'.', b'I', b'n', b'c', b'.',
];
const PACKET_SIZE: usize = 640;
const DEV_PAGE: u8 = 0x5e;
pub const VENDOR_ID: u16 = 0x0b05;
pub const PROD_ID: u16 = 0x193b;

/// `get_anime_type` is very broad, matching on part of the laptop board name only. For this
/// reason `find_node()` must be used also to verify if the USB device is available.
///
/// The currently known USB device is `19b6`.
#[inline]
pub fn get_anime_type() -> Result<AnimeType, AnimeError> {
    let dmi = sysfs_class::DmiId::default();
    let board_name = dmi.board_name()?;

    if board_name.contains("GA401Q") {
        return Ok(AnimeType::GA401);
    } else if board_name.contains("GA401R") {
        return Ok(AnimeType::GA402);
    }
    Err(AnimeError::UnsupportedDevice)
}

/// Find the USB device node - known devices so far: `19b6`
#[inline]
pub fn find_node(id_product: &str) -> Result<String, AnimeError> {
    let mut enumerator =
        udev::Enumerator::new().map_err(|err| AnimeError::Udev("enumerator failed".into(), err))?;
    enumerator
        .match_subsystem("usb")
        .map_err(|err| AnimeError::Udev("match_subsystem failed".into(), err))?;

    for device in enumerator
        .scan_devices()
        .map_err(|err| AnimeError::Udev("scan_devices failed".into(), err))?
    {
        if let Some(attr) = device.attribute_value("idProduct") {
            if attr == id_product {
                if let Some(dev_node) = device.devnode() {
                    return Ok(dev_node.to_string_lossy().to_string());
                }
            }
        }
    }
    Err(AnimeError::NoDevice)
}

/// Get the two device initialization packets. These are required for device start
/// after the laptop boots.
#[inline]
pub const fn pkts_for_init() -> [[u8; PACKET_SIZE]; 2] {
    let mut packets = [[0; PACKET_SIZE]; 2];
    packets[0][0] = DEV_PAGE; // This is the USB page we're using throughout
    let mut count = 0;
    while count < INIT_STR.len() {
        packets[0][count] = INIT_STR[count];
        count += 1;
    }
    //
    packets[1][0] = DEV_PAGE; // write it to be sure?
    packets[1][1] = 0xc2;
    packets
}

/// Should be written to the device after writing the two main data packets that
/// make up the display data packet
#[inline]
pub const fn pkt_for_flush() -> [u8; PACKET_SIZE] {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xc0;
    pkt[2] = 0x03;
    pkt
}

/// Get the packet required for setting the device to on, on boot. Requires
/// pkt_for_apply()` to be written after.
#[inline]
pub const fn pkt_for_set_boot(status: bool) -> [u8; PACKET_SIZE] {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xc3;
    pkt[2] = 0x01;
    pkt[3] = if status { 0x00 } else { 0x80 };
    pkt
}

/// Get the packet required for setting the device to on. Requires `pkt_for_apply()`
/// to be written after.
#[inline]
pub const fn pkt_for_set_on(on: bool) -> [u8; PACKET_SIZE] {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xc0;
    pkt[2] = 0x04;
    pkt[3] = if on { 0x03 } else { 0x00 };
    pkt
}

/// Packet required to apply a device setting
#[inline]
pub const fn pkt_for_apply() -> [u8; PACKET_SIZE] {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xc4;
    pkt[2] = 0x01;
    pkt[3] = 0x80;
    pkt
}
