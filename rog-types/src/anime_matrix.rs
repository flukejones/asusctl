pub const WIDTH: usize = 34; // Width is definitely 34 items
pub const HEIGHT: usize = 56;
pub type AniMeBufferType = [[u8; WIDTH]; HEIGHT];
pub type AniMePacketType = [[u8; 640]; 2];
const BLOCK_START: usize = 7;
const BLOCK_END: usize = 634;

pub const ANIME_PANE1_PREFIX: [u8; 7] = [0x5e, 0xc0, 0x02, 0x01, 0x00, 0x73, 0x02];
pub const ANIME_PANE2_PREFIX: [u8; 7] = [0x5e, 0xc0, 0x02, 0x74, 0x02, 0x73, 0x02];

/// Helper structure for writing images.
///
///  See the examples for ways to write an image to `AniMeMatrix` format.
pub struct AniMeMatrix(AniMeBufferType);

impl Default for AniMeMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl AniMeMatrix {
    pub fn new() -> Self {
        AniMeMatrix([[0u8; WIDTH]; HEIGHT])
    }

    pub fn get(&self) -> &AniMeBufferType {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut AniMeBufferType {
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

impl From<AniMeMatrix> for AniMePacketType {
    /// Do conversion from the nested Vec in AniMeMatrix to the two required
    /// packets suitable for sending over USB
    #[inline]
    fn from(anime: AniMeMatrix) -> Self {
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
    use crate::anime_matrix::{AniMeMatrix, AniMePacketType};

    #[test]
    fn check_data_alignment() {
        let mut matrix = AniMeMatrix::new();
        {
            let tmp = matrix.get_mut();
            for row in tmp.iter_mut() {
                row[row.len() - 1] = 0xff;
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
