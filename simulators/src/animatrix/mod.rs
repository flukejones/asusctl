use rog_anime::AnimeType;

use self::map_ga401::GA401;
use self::map_ga402::GA402;
use self::map_gu604::GU604;

mod map_ga401;
mod map_ga402;
mod map_gu604;

#[derive(Clone, Copy)]
pub struct Row(
    /// The USB packet index, this is mapped to the 4th byte (idx = 3) and is
    /// one of (in order of packets): 1. `0x01`
    /// 2. `0x74`
    /// 3. `0xe7`
    pub u8,
    /// Starting index in that packet
    pub usize,
    /// The length to read inclusive
    pub usize,
    /// Offset to the right by how many LEDs
    pub i32,
);

#[derive(Clone, Copy)]
pub struct LedShape {
    /// Vertical offset from center for the top/bottom points
    pub vertical: i32,
    /// Horizontal offset from center for the top/bottom points
    pub horizontal: i32,
}

pub struct AniMatrix {
    rows: Vec<Row>,
    led_shape: LedShape,
}

impl AniMatrix {
    pub fn new(model: AnimeType) -> Self {
        let led_shape = match model {
            AnimeType::GA401 => LedShape {
                vertical: 2,
                horizontal: 5,
            },
            AnimeType::GA402 | AnimeType::Unknown => LedShape {
                vertical: 2,
                horizontal: 5,
            },
            AnimeType::GU604 => LedShape {
                vertical: 2,
                horizontal: 5,
            },
        };

        // Do a hard mapping of each (derived from wireshardk captures)
        let rows = match model {
            AnimeType::GA401 => GA401.to_vec(),
            AnimeType::GA402 | AnimeType::Unknown => GA402.to_vec(),
            AnimeType::GU604 => GU604.to_vec(),
        };

        Self { rows, led_shape }
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn led_shape(&self) -> LedShape {
        self.led_shape
    }
}
