use std::convert::TryFrom;
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};

use dmi_id::DMIID;
use log::info;
use serde::{Deserialize, Serialize};
#[cfg(feature = "dbus")]
use zbus::zvariant::{OwnedValue, Type, Value};

use crate::error::{AnimeError, Result};
use crate::usb::{AnimAwake, AnimBooting, AnimShutdown, AnimSleeping, Brightness};
use crate::{AnimTime, AnimeGif};

/// The first 7 bytes of a USB packet are accounted for by `USB_PREFIX1` and
/// `USB_PREFIX2`
const BLOCK_START: usize = 7;
/// *Not* inclusive, the byte before this is the final for each "pane"
const BLOCK_END: usize = 634;
/// Individual usable data length of each USB packet
const PANE_LEN: usize = BLOCK_END - BLOCK_START;

/// First packet is for GA401 + GA402
pub const USB_PREFIX1: [u8; 7] = [
    0x5e, 0xc0, 0x02, 0x01, 0x00, 0x73, 0x02,
];
/// Second packet is for GA401 + GA402
pub const USB_PREFIX2: [u8; 7] = [
    0x5e, 0xc0, 0x02, 0x74, 0x02, 0x73, 0x02,
];
/// Third packet is for GA402 matrix
pub const USB_PREFIX3: [u8; 7] = [
    0x5e, 0xc0, 0x02, 0xe7, 0x04, 0x73, 0x02,
];

#[cfg_attr(feature = "dbus", derive(Type, Value, OwnedValue))]
#[derive(Default, Deserialize, PartialEq, Eq, Clone, Copy, Serialize, Debug)]
pub struct Animations {
    pub boot: AnimBooting,
    pub awake: AnimAwake,
    pub sleep: AnimSleeping,
    pub shutdown: AnimShutdown,
}

// TODO: move this out
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub struct DeviceState {
    pub display_enabled: bool,
    pub display_brightness: Brightness,
    pub builtin_anims_enabled: bool,
    pub builtin_anims: Animations,
    pub off_when_unplugged: bool,
    pub off_when_suspended: bool,
    pub off_when_lid_closed: bool,
    pub brightness_on_battery: Brightness,
}

#[cfg_attr(feature = "dbus", derive(Type), zvariant(signature = "s"))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum AnimeType {
    GA401,
    GA402,
    GU604,
    #[default]
    Unsupported,
}

impl FromStr for AnimeType {
    type Err = AnimeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "ga401" | "GA401" => Self::GA401,
            "ga402" | "GA402" => Self::GA402,
            "gu604" | "GU604" => Self::GU604,
            _ => Self::Unsupported,
        })
    }
}

impl AnimeType {
    pub fn from_dmi() -> Self {
        let board_name = DMIID::new().unwrap_or_default().board_name.to_uppercase();
        if board_name.contains("GA401I") || board_name.contains("GA401Q") {
            AnimeType::GA401
        } else if board_name.contains("GA402R") || board_name.contains("GA402X") {
            AnimeType::GA402
        } else if board_name.contains("GU604V") {
            AnimeType::GU604
        } else {
            AnimeType::Unsupported
        }
    }

    /// The width of diagonal images
    pub fn width(&self) -> usize {
        match self {
            AnimeType::GU604 => 70,
            _ => 74,
        }
    }

    /// The height of diagonal images
    pub fn height(&self) -> usize {
        match self {
            AnimeType::GA401 => 36,
            AnimeType::GU604 => 43,
            _ => 39,
        }
    }

    /// The length of usable data for this type
    pub fn data_length(&self) -> usize {
        match self {
            AnimeType::GA401 => PANE_LEN * 2,
            AnimeType::GU604 => PANE_LEN * 3,
            _ => PANE_LEN * 3,
        }
    }
}

/// The minimal serializable data that can be transferred over wire types.
/// Other data structures in `rog_anime` will convert to this.
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimeDataBuffer {
    data: Vec<u8>,
    anime: AnimeType,
}

impl AnimeDataBuffer {
    #[inline]
    pub fn new(anime: AnimeType) -> Self {
        let len = anime.data_length();

        AnimeDataBuffer {
            data: vec![0u8; len],
            anime,
        }
    }

    /// Get the inner data buffer
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable slice of the inner buffer
    #[inline]
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Create from a vector of bytes
    ///
    /// # Errors
    /// Will error if the vector length is not `ANIME_DATA_LEN`
    #[inline]
    pub fn from_vec(anime: AnimeType, data: Vec<u8>) -> Result<Self> {
        if data.len() != anime.data_length() {
            return Err(AnimeError::DataBufferLength);
        }

        Ok(Self { data, anime })
    }
}

/// The packets to be written to USB
pub type AnimePacketType = Vec<[u8; 640]>;

impl TryFrom<AnimeDataBuffer> for AnimePacketType {
    type Error = AnimeError;

    fn try_from(anime: AnimeDataBuffer) -> std::result::Result<Self, Self::Error> {
        if anime.data.len() != anime.anime.data_length() {
            return Err(AnimeError::DataBufferLength);
        }

        let mut buffers = match anime.anime {
            AnimeType::GA401 => vec![[0; 640]; 2],
            AnimeType::GA402 | AnimeType::GU604 | AnimeType::Unsupported => vec![[0; 640]; 3],
        };

        for (idx, chunk) in anime.data.as_slice().chunks(PANE_LEN).enumerate() {
            buffers[idx][BLOCK_START..BLOCK_END].copy_from_slice(chunk);
        }
        buffers[0][..7].copy_from_slice(&USB_PREFIX1);
        buffers[1][..7].copy_from_slice(&USB_PREFIX2);

        if matches!(
            anime.anime,
            AnimeType::GA402 | AnimeType::GU604 | AnimeType::Unsupported
        ) {
            buffers[2][..7].copy_from_slice(&USB_PREFIX3);
        }
        Ok(buffers)
    }
}

/// This runs the animations as a blocking loop by using the `callback` to write
/// data
///
/// If `callback` is `Ok(true)` then `run_animation` will exit the animation
/// loop early.
pub fn run_animation(frames: &AnimeGif, callback: &dyn Fn(AnimeDataBuffer) -> Result<bool>) {
    let mut count = 0;
    let start = Instant::now();

    let mut timed = false;
    let mut run_time = frames.total_frame_time();
    if let AnimTime::Fade(time) = frames.duration() {
        if let Some(middle) = time.show_for() {
            run_time = middle + time.total_fade_time();
        }
        // add a small buffer
        run_time += Duration::from_millis(250);
        timed = true;
    } else if let AnimTime::Time(time) = frames.duration() {
        run_time = time;
        timed = true;
    }

    // After setting up all the data
    let mut fade_in = Duration::from_millis(0);
    let mut fade_out = Duration::from_millis(0);
    let mut fade_in_step = 0.0;
    let mut fade_in_accum = 0.0;
    let mut fade_out_step = 0.0;
    let mut fade_out_accum;
    if let AnimTime::Fade(time) = frames.duration() {
        fade_in = time.fade_in();
        fade_out = time.fade_out();
        fade_in_step = 1.0 / fade_in.as_secs_f32();
        fade_out_step = 1.0 / fade_out.as_secs_f32();

        if time.total_fade_time() > run_time {
            println!("Total fade in/out time larger than gif run time. Setting fades to half");
            fade_in = run_time / 2;
            fade_in_step = 1.0 / (run_time / 2).as_secs_f32();

            fade_out = run_time / 2;
            fade_out_step = 1.0 / (run_time / 2).as_secs_f32();
        }
    }

    'animation: loop {
        for frame in frames.frames() {
            let frame_start = Instant::now();
            let mut output = frame.frame().clone();

            if let AnimTime::Fade(_) = frames.duration() {
                if frame_start <= start + fade_in {
                    for pixel in output.data_mut() {
                        *pixel = (*pixel as f32 * fade_in_accum) as u8;
                    }
                    fade_in_accum = fade_in_step * (frame_start - start).as_secs_f32();
                } else if frame_start > (start + run_time) - fade_out {
                    if run_time > (frame_start - start) {
                        fade_out_accum =
                            fade_out_step * (run_time - (frame_start - start)).as_secs_f32();
                    } else {
                        fade_out_accum = 0.0;
                    }
                    for pixel in output.data_mut() {
                        *pixel = (*pixel as f32 * fade_out_accum) as u8;
                    }
                }
            }

            // TODO: Log this error
            if matches!(callback(output), Ok(true)) {
                info!("rog-anime: animation frame-loop callback asked to exit early");
                return;
            }

            if timed && Instant::now().duration_since(start) > run_time {
                break 'animation;
            }
            sleep(frame.delay());
        }
        if let AnimTime::Count(times) = frames.duration() {
            count += 1;
            if count >= times {
                break 'animation;
            }
        }
    }
}
