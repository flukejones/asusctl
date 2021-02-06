use serde_derive::{Deserialize, Serialize};
use zvariant_derive::Type;

pub const WIDTH: usize = 34; // Width is definitely 34 items
pub const HEIGHT: usize = 56;
pub type AniMePacketType = [[u8; 640]; 2];
const BLOCK_START: usize = 7;
/// *Not* inclusive, the byte before this is the final for each "pane"
const BLOCK_END: usize = 634;
pub const PANE_LEN: usize = BLOCK_END - BLOCK_START;
/// The length of usable data
pub const FULL_PANE_LEN: usize = PANE_LEN * 2;

pub const ANIME_PANE1_PREFIX: [u8; 7] = [0x5e, 0xc0, 0x02, 0x01, 0x00, 0x73, 0x02];
pub const ANIME_PANE2_PREFIX: [u8; 7] = [0x5e, 0xc0, 0x02, 0x74, 0x02, 0x73, 0x02];

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct AniMeDataBuffer(Vec<u8>);

impl Default for AniMeDataBuffer {
     fn default() -> Self {
          Self::new()
       }
    }
    
impl AniMeDataBuffer {
    pub fn new() -> Self {
        AniMeDataBuffer(vec![0u8; FULL_PANE_LEN])
    }

    pub fn get(&self) -> &[u8] {
        &self.0
    }

    pub fn set(&mut self, input: [u8; FULL_PANE_LEN]) {
        self.0 = input.to_vec();
    }
}

impl From<AniMeDataBuffer> for AniMePacketType {
    #[inline]
    fn from(anime: AniMeDataBuffer) -> Self {
        assert!(anime.0.len() == FULL_PANE_LEN);
        let mut buffers = [[0; 640]; 2];
        for (idx, chunk) in anime.0.as_slice().chunks(PANE_LEN).enumerate() {
            buffers[idx][BLOCK_START..BLOCK_END].copy_from_slice(chunk);
        }
        buffers
    }
}

/// Helper structure for writing images.
///
///  See the examples for ways to write an image to `AniMeMatrix` format.
#[derive(Debug, Deserialize, Serialize, Type)]
pub struct AniMeImageBuffer(Vec<Vec<u8>>);

impl Default for AniMeImageBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl AniMeImageBuffer {
    pub fn new() -> Self {
        AniMeImageBuffer(vec![vec![0u8; WIDTH]; HEIGHT])
    }

    pub fn get(&self) -> &Vec<Vec<u8>> {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut Vec<Vec<u8>> {
        &mut self.0
    }

    pub fn fill_with(&mut self, fill: u8) {
        for row in self.0.iter_mut() {
            for x in row.iter_mut() {
                *x = fill;
            }
        }
    }

    pub fn debug_print(&self) {
        // this is the index from right. It is used to progressively shorten rows
        let mut prog_row_len = WIDTH - 2;

        for (count, row) in self.0.iter().enumerate() {
            // Write the top block of LEDs (first 7 rows)
            if count < 6 {
                if count % 2 != 0 {
                    print!(" ");
                } else {
                    print!("");
                }
                let tmp = if count == 0 || count == 1 || count == 3 || count == 5 {
                    row[1..].iter()
                } else {
                    row.iter()
                };
                for _ in tmp {
                    print!(" XY");
                }

                println!();
            } else {
                // Switch to next block (looks like )
                if count % 2 != 0 {
                    // Row after 6 is only 1 less, then rows after 7 follow pattern
                    if count == 7 {
                        prog_row_len -= 1;
                    } else {
                        prog_row_len -= 2;
                    }
                } else {
                    prog_row_len += 1; // if count 6, 0
                }

                let index = row.len() - prog_row_len;

                if count % 2 == 0 {
                    print!(" ");
                }
                for (i, _) in row.iter().enumerate() {
                    if i >= index {
                        print!(" XY");
                    } else {
                        print!("   ");
                    }
                }
                println!();
            }
        }
    }
}

impl From<AniMeImageBuffer> for AniMePacketType {
    /// Do conversion from the nested Vec in AniMeMatrix to the two required
    /// packets suitable for sending over USB
    #[inline]
    fn from(anime: AniMeImageBuffer) -> Self {
        let mut buffers = [[0; 640]; 2];

        let mut write_index = BLOCK_START;
        let mut write_block = &mut buffers[0];
        let mut block1_done = false;

        // this is the index from right. It is used to progressively shorten rows
        let mut prog_row_len = WIDTH - 2;

        for (count, row) in anime.0.iter().enumerate() {
            // Write the top block of LEDs (first 7 rows)
            if count < 6 {
                for (i, x) in row.iter().enumerate() {
                    // Rows 0, 1, 3, 5 are short and misaligned
                    if count == 0 || count == 1 || count == 3 || count == 5 {
                        if i > 0 {
                            write_block[write_index - 1] = *x;
                        }
                    } else {
                        write_block[write_index] = *x;
                    }
                    write_index += 1;
                }
            } else {
                // Switch to next block (looks like )
                if count % 2 != 0 {
                    // Row after 6 is only 1 less, then rows after 7 follow pattern
                    if count == 7 {
                        prog_row_len -= 1;
                    } else {
                        prog_row_len -= 2;
                    }
                } else {
                    prog_row_len += 1; // if count 6, 0
                }

                let index = row.len() - prog_row_len;
                for n in row.iter().skip(index) {
                    // Require a special case to catch the correct end-of-packet which is
                    // 6 bytes from the end
                    if write_index == BLOCK_END && !block1_done {
                        block1_done = true;
                        write_block = &mut buffers[1];
                        write_index = BLOCK_START;
                    }

                    write_block[write_index] = *n;
                    write_index += 1;
                }
            }
        }
        buffers
    }
}

#[cfg(test)]
mod tests {
    use crate::anime_matrix::*;

    use super::AniMeDataBuffer;

    #[test]
    fn check_from_data_buffer() {
        let mut data = AniMeDataBuffer::new();
        data.set([42u8; FULL_PANE_LEN]);

        let out: AniMePacketType = data.into();
    }

    #[test]
    fn check_data_alignment() {
        let mut matrix = AniMeImageBuffer::new();
        {
            let tmp = matrix.get_mut();
            for row in tmp.iter_mut() {
                let idx = row.len() - 1;
                row[idx] = 0xff;
            }
        }

        let matrix: AniMePacketType = AniMePacketType::from(matrix);

        // The bytes at the right of the initial AniMeMatrix should always end up aligned in the
        // same place after conversion to data packets

        // Check against manually worked out right align
        assert_eq!(
            matrix[0].to_vec(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
            .to_vec()
        );
        assert_eq!(
            matrix[1].to_vec(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0,
                0, 0, 0, 0
            ]
            .to_vec()
        );
    }
}
