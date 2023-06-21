//! Utils for writing to the `AniMe` USB device
//!
//! Use of the device requires a few steps:
//! 1. Initialise the device by writing the two packets from
//! `get_init_packets()` 2. Write data from `AnimePacketType`
//! 3. Write the packet from `get_flush_packet()`, which tells the device to
//! display the data from step 2
//!
//! Step 1 need to applied only on fresh system boot.

use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "dbus")]
use zbus::zvariant::Type;

use crate::error::AnimeError;
use crate::AnimeType;

const PACKET_SIZE: usize = 640;
const DEV_PAGE: u8 = 0x5e;
pub const VENDOR_ID: u16 = 0x0b05;
pub const PROD_ID: u16 = 0x193b;

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
/// Base LED brightness of the display
pub enum Brightness {
    Off,
    Low,
    #[default]
    Med,
    High,
}

impl FromStr for Brightness {
    type Err = AnimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Off" | "off" => Brightness::Off,
            "Low" | "low" => Brightness::Low,
            "Med" | "med" => Brightness::Med,
            "High" | "high" => Brightness::High,
            _ => Brightness::Med,
        })
    }
}

impl From<u8> for Brightness {
    fn from(v: u8) -> Brightness {
        match v {
            0 => Brightness::Off,
            2 => Brightness::Low,
            3 => Brightness::High,
            _ => Brightness::Med,
        }
    }
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub enum AnimBooting {
    #[default]
    GlitchConstruction,
    StaticEmergence,
}

impl FromStr for AnimBooting {
    type Err = AnimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GlitchConstruction" => Ok(Self::GlitchConstruction),
            "StaticEmergence" => Ok(Self::StaticEmergence),
            _ => Err(AnimeError::ParseError(s.to_owned())),
        }
    }
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub enum AnimAwake {
    #[default]
    BinaryBannerScroll,
    RogLogoGlitch,
}

impl FromStr for AnimAwake {
    type Err = AnimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BinaryBannerScroll" => Ok(Self::BinaryBannerScroll),
            "RogLogoGlitch" => Ok(Self::RogLogoGlitch),
            _ => Err(AnimeError::ParseError(s.to_owned())),
        }
    }
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub enum AnimSleeping {
    #[default]
    BannerSwipe,
    Starfield,
}

impl FromStr for AnimSleeping {
    type Err = AnimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BannerSwipe" => Ok(Self::BannerSwipe),
            "Starfield" => Ok(Self::Starfield),
            _ => Err(AnimeError::ParseError(s.to_owned())),
        }
    }
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub enum AnimShutdown {
    #[default]
    GlitchOut,
    SeeYa,
}

impl FromStr for AnimShutdown {
    type Err = AnimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GlitchOut" => Ok(Self::GlitchOut),
            "SeeYa" => Ok(Self::SeeYa),
            _ => Err(AnimeError::ParseError(s.to_owned())),
        }
    }
}

/// `get_anime_type` is very broad, matching on part of the laptop board name
/// only. For this reason `find_node()` must be used also to verify if the USB
/// device is available.
///
/// The currently known USB device is `19b6`.
#[inline]
pub fn get_anime_type() -> Result<AnimeType, AnimeError> {
    let dmi = sysfs_class::DmiId::default();
    let board_name = dmi.board_name()?;

    if board_name.contains("GA401I") || board_name.contains("GA401Q") {
        return Ok(AnimeType::GA401);
    } else if board_name.contains("GA402R") {
        return Ok(AnimeType::GA402);
    } else if board_name.contains("GU604V") {
        return Ok(AnimeType::GU604);
    }
    log::warn!("AniMe Matrix device found but not yet supported");
    Ok(AnimeType::Unknown)
}

/// Get the two device initialization packets. These are required for device
/// start after the laptop boots.
#[inline]
pub const fn pkts_for_init() -> [[u8; PACKET_SIZE]; 2] {
    let mut packets = [[0; PACKET_SIZE]; 2];
    packets[0][0] = DEV_PAGE; // This is the USB page we're using throughout
    let mut count = 0;
    // TODO: memcpy or slice copy
    let bytes = "ASUS Tech.Inc.".as_bytes();
    while count < bytes.len() {
        packets[0][count + 1] = bytes[count];
        count += 1;
    }
    //
    packets[1][0] = DEV_PAGE;
    packets[1][1] = 0xc2;
    packets
}

/// Should be written to the device after writing the two main data packets that
/// make up the display data packet
#[inline]
pub const fn pkt_flush() -> [u8; PACKET_SIZE] {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xc0;
    pkt[2] = 0x03;
    pkt
}

/// Packet for setting the brightness (0-3). Requires
/// `pkt_for_apply()` to be written after.
#[inline]
pub const fn pkt_set_brightness(brightness: Brightness) -> [u8; PACKET_SIZE] {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xc0;
    pkt[2] = 0x04;
    pkt[3] = brightness as u8;
    pkt
}

/// Enable the display?
#[inline]
pub const fn pkt_set_enable_display(status: bool) -> [u8; PACKET_SIZE] {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xc3;
    pkt[2] = 0x01;
    pkt[3] = if status { 0x00 } else { 0x80 };
    pkt
}

/// Enable builtin animations?
#[inline]
pub const fn pkt_set_enable_powersave_anim(status: bool) -> [u8; PACKET_SIZE] {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xc4;
    pkt[2] = 0x01;
    pkt[3] = if status { 0x00 } else { 0x80 };
    pkt
}

/// Set which animations are shown for each stage
#[inline]
pub const fn pkt_set_builtin_animations(
    boot: AnimBooting,
    awake: AnimAwake,
    sleep: AnimSleeping,
    shutdown: AnimShutdown,
) -> [u8; PACKET_SIZE] {
    let mut pkt = [0; PACKET_SIZE];
    pkt[0] = DEV_PAGE;
    pkt[1] = 0xc5;
    pkt[2] = (awake as u8)
        | ((sleep as u8) << 0x01)
        | ((shutdown as u8) << 0x02)
        | ((boot as u8) << 0x03);
    pkt
}
