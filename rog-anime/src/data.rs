use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::{Duration, Instant},
};

use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "dbus")]
use zvariant_derive::Type;

use crate::{error::AnimeError, AnimTime, AnimeGif};

/// The first 7 bytes of a USB packet are accounted for by `USB_PREFIX1` and `USB_PREFIX2`
const BLOCK_START: usize = 7;
/// *Not* inclusive, the byte before this is the final for each "pane"
const BLOCK_END: usize = 634;
/// Individual usable data length of each USB packet
const PANE_LEN: usize = BLOCK_END - BLOCK_START;
/// The length of usable data
pub const ANIME_DATA_LEN: usize = PANE_LEN * 2;

const USB_PREFIX1: [u8; 7] = [0x5e, 0xc0, 0x02, 0x01, 0x00, 0x73, 0x02];
const USB_PREFIX2: [u8; 7] = [0x5e, 0xc0, 0x02, 0x74, 0x02, 0x73, 0x02];

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, PartialEq, Copy, Clone, Deserialize, Serialize)]

pub struct AnimePowerStates {
    pub enabled: bool,
    pub boot_anim_enabled: bool,
}

/// The minimal serializable data that can be transferred over wire types.
/// Other data structures in `rog_anime` will convert to this.
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimeDataBuffer(Vec<u8>);

impl Default for AnimeDataBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimeDataBuffer {
    #[inline]
    pub fn new() -> Self {
        AnimeDataBuffer(vec![0u8; ANIME_DATA_LEN])
    }

    /// Get the inner data buffer
    #[inline]
    pub fn get(&self) -> &[u8] {
        &self.0
    }

    /// Get a mutable slice of the inner buffer
    #[inline]
    pub fn get_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    /// Create from a vector of bytes
    ///
    /// # Panics
    /// Will panic if the vector length is not `ANIME_DATA_LEN`
    #[inline]
    pub fn from_vec(input: Vec<u8>) -> Self {
        assert_eq!(input.len(), ANIME_DATA_LEN);
        Self(input)
    }
}

/// The two packets to be written to USB
pub type AnimePacketType = [[u8; 640]; 2];

impl From<AnimeDataBuffer> for AnimePacketType {
    #[inline]
    fn from(anime: AnimeDataBuffer) -> Self {
        assert!(anime.0.len() == ANIME_DATA_LEN);
        let mut buffers = [[0; 640]; 2];
        for (idx, chunk) in anime.0.as_slice().chunks(PANE_LEN).enumerate() {
            buffers[idx][BLOCK_START..BLOCK_END].copy_from_slice(chunk);
        }
        buffers[0][..7].copy_from_slice(&USB_PREFIX1);
        buffers[1][..7].copy_from_slice(&USB_PREFIX2);
        buffers
    }
}

/// This runs the animations as a blocking loop by using the `callback` to write data
pub fn run_animation(
    frames: &AnimeGif,
    do_early_return: Arc<AtomicBool>,
    callback: &dyn Fn(AnimeDataBuffer),
) -> Result<(), AnimeError> {
    let mut count = 0;
    let start = Instant::now();

    let mut timed = false;
    let mut run_time = frames.total_frame_time();
    println!("Real gif run length = {:?}", run_time);
    if let AnimTime::Fade(time) = frames.duration() {
        if let Some(middle) = time.show_for() {
            run_time = middle + time.total_fade_time();
        }
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
            if do_early_return.load(Ordering::SeqCst) {
                return Ok(());
            }
            let mut output = frame.frame().clone();

            if let AnimTime::Fade(_) = frames.duration() {
                if frame_start <= start + fade_in {
                    for pixel in output.get_mut() {
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
                    for pixel in output.get_mut() {
                        *pixel = (*pixel as f32 * fade_out_accum) as u8;
                    }
                }
            }

            callback(output);

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
    Ok(())
}
