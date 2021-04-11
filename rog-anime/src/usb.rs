//! Utils for writing to the AniMe USB device
//!
//! Use of the device requires a few steps:
//! 1. Initialise the device by writing the two packets from `get_init_packets()`
//! 2. Write data from `AnimePacketType`
//! 3. Write the packet from `get_flush_packet()`, which tells the device to display the data from step 2
//!
//! Step 1 need to applied only on fresh system boot.

const INIT_STR: [u8; 15] = [
    0x5e, b'A', b'S', b'U', b'S', b' ', b'T', b'E', b'C', b'H', b'.', b'I', b'N', b'C', b'.',
];
const PACKET_SIZE: usize = 640;
const DEV_PAGE: u8 = 0x5e;
pub const VENDOR_ID: u16 = 0x0b05;
pub const PROD_ID: u16 = 0x193b;

/// Get the two device initialization packets. These are required for device start
/// after the laptop boots.
#[inline]
pub const fn pkts_for_init() -> [[u8; PACKET_SIZE]; 2] {
    let mut packets = [[0; PACKET_SIZE]; 2];
    packets[0][0] = DEV_PAGE; // This is the USB page we're using throughout
    let mut count = 0;
    while count < INIT_STR.len() {
        packets[0][count] = INIT_STR[count];
        count +=1;
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
