//! Utils for writing to the `Slash` USB device
//!
//! Use of the device requires a few steps:
//! 1. Initialise the device by writing the two packets from
//! `get_init_packets()` 2. Write data from `SLashPacketType`
//! 3. Write the packet from `get_flush_packet()`, which tells the device to
//! display the data from step 2
//!
//! Step 1 needs to be applied only on fresh system boot.

use dmi_id::DMIID;

#[cfg(feature = "dbus")]
use crate::error::SlashError;
use crate::{SlashMode, SlashType};

const PACKET_SIZE: usize = 128;
const DEV_PAGE: u8 = 0x5e;
pub const VENDOR_ID: u16 = 0x0b05;
pub const PROD_ID: u16 = 0x193b;
pub const PROD_ID_STR: &str = "193B";

pub type SlashUsbPacket = [u8; PACKET_SIZE];

/// `get_anime_type` is very broad, matching on part of the laptop board name
/// only. For this reason `find_node()` must be used also to verify if the USB
/// device is available.
///
/// The currently known USB device is `193B`.
#[inline]
pub fn get_maybe_slash_type() -> Result<SlashType, SlashError> {
    let dmi = DMIID::new().map_err(|_| SlashError::NoDevice)?; // TODO: better error
    let board_name = dmi.board_name;

    if board_name.contains("GA403") {
        return Ok(SlashType::GA403);
    } else if board_name.contains("GA605") {
        return Ok(SlashType::GA605);
    } else if board_name.contains("GU605") {
        return Ok(SlashType::GU605);
    }
    Ok(SlashType::Unsupported)
}

/// Get the two device initialization packets. These are required for device
/// start after the laptop boots.
#[inline]
pub const fn pkts_for_init() -> [SlashUsbPacket; 2] {
    let mut pkt1 = [0; PACKET_SIZE];
    pkt1[0] = DEV_PAGE;
    pkt1[1] = 0xd7;
    pkt1[2] = 0x00;
    pkt1[3] = 0x00;
    pkt1[4] = 0x01;
    pkt1[5] = 0xac;

    let mut pkt2 = [0; PACKET_SIZE];
    pkt2[0] = DEV_PAGE;
    pkt2[1] = 0xd2;
    pkt2[2] = 0x02;
    pkt2[3] = 0x01;
    pkt2[4] = 0x08;
    pkt2[5] = 0xab;

    [pkt1, pkt2]
}

#[inline]
pub const fn pkt_save() -> SlashUsbPacket {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xd4;
    pkt[2] = 0x00;
    pkt[3] = 0x00;
    pkt[4] = 0x01;
    pkt[5] = 0xab;

    pkt
}

#[inline]
pub const fn pkt_set_mode(mode: SlashMode) -> [SlashUsbPacket; 2] {
    let mut pkt1 = [0; PACKET_SIZE];
    pkt1[0] = DEV_PAGE;
    pkt1[1] = 0x02;
    pkt1[2] = 0x03;
    pkt1[3] = 0x00;
    pkt1[4] = 0x0c;

    let mut pkt2 = [0; PACKET_SIZE];
    pkt2[0] = DEV_PAGE;
    pkt2[1] = 0xd3;
    pkt2[2] = 0x04;
    pkt2[3] = 0x00;
    pkt2[4] = 0x0c;
    pkt2[5] = 0x01;
    pkt2[6] = mode as u8;
    pkt2[7] = 0x02;
    pkt2[8] = 0x19;
    pkt2[9] = 0x03;
    pkt2[10] = 0x13;
    pkt2[11] = 0x04;
    pkt2[12] = 0x11;
    pkt2[13] = 0x05;
    pkt2[14] = 0x12;
    pkt2[15] = 0x06;
    pkt2[16] = 0x13;

    [pkt1, pkt2]
}

#[inline]
pub const fn pkt_set_options(enabled: bool, brightness: u8, interval: u8) -> SlashUsbPacket {
    let status_byte = if enabled { 0x01 } else { 0x00 };

    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xd3;
    pkt[2] = 0x03;
    pkt[3] = 0x01;
    pkt[4] = 0x08;
    pkt[5] = 0xab;
    pkt[6] = 0xff;
    pkt[7] = 0x01;
    pkt[8] = status_byte;
    pkt[9] = 0x06;
    pkt[10] = brightness;
    pkt[11] = 0xff;
    pkt[12] = interval;

    pkt
}
