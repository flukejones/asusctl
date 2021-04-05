use serde_derive::{Deserialize, Serialize};
use std::{fs::File, path::{Path}, time::Duration};

use crate::{error::AnimeError, AniMeDataBuffer, AniMeDiagonal};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AniMeFrame {
    data: AniMeDataBuffer,
    delay: Duration,
}

impl AniMeFrame {
    pub fn frame(&self) -> &AniMeDataBuffer {
        &self.data
    }

    pub fn delay(&self) -> Duration {
        self.delay
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AniMeGif(Vec<AniMeFrame>);

impl AniMeGif {
    pub fn new(file_name: &Path, brightness: f32) -> Result<Self, AnimeError> {
        let mut frames = Vec::new();
        let mut matrix = AniMeDiagonal::new();

        let mut decoder = gif::DecodeOptions::new();
        // Configure the decoder such that it will expand the image to RGBA.
        decoder.set_color_output(gif::ColorOutput::RGBA);
        // Read the file header
        let file = File::open(file_name)?;
        let mut decoder = decoder.read_info(file)?;

        while let Some(frame) = decoder.read_next_frame()? {
            let wait = frame.delay * 10;
            for (y, row) in frame.buffer.chunks(frame.width as usize * 4).enumerate() {
                for (x, px) in row.chunks(4).enumerate() {
                    if px[3] != 255 {
                        // should be t but not in some gifs? What, ASUS, what?
                        continue;
                    }
                    matrix.get_mut()[y + frame.top as usize][x + frame.left as usize] =
                        (px[0] as f32 * brightness) as u8;
                }
            }

            frames.push(AniMeFrame {
                data: <AniMeDataBuffer>::from(&matrix),
                delay: Duration::from_millis(wait as u64),
            });
        }
        Ok(Self(frames))
    }

    pub fn frames(&self) -> &[AniMeFrame] {
        &self.0
    }
}
