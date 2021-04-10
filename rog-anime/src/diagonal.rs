use std::{path::Path, time::Duration};

use crate::{
    data::{AnimeDataBuffer, ANIME_DATA_LEN},
    error::AnimeError,
};

const WIDTH: usize = 74;
const HEIGHT: usize = 36;

/// Mostly intended to be used with ASUS gifs, but can be used for other purposes (like images)
#[derive(Debug, Clone)]
pub struct AnimeDiagonal([[u8; WIDTH]; HEIGHT], Option<Duration>);

impl Default for AnimeDiagonal {
    #[inline]
    fn default() -> Self {
        Self::new(None)
    }
}

impl AnimeDiagonal {
    #[inline]
    pub fn new(duration: Option<Duration>) -> Self {
        Self([[0u8; WIDTH]; HEIGHT], duration)
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut [[u8; WIDTH]; HEIGHT] {
        &mut self.0
    }

    /// Get a full diagonal row where `x` `y` is the starting point  and `len` is the length of data.
    fn get_row(&self, x: usize, y: usize, len: usize) -> Vec<u8> {
        let mut buf = Vec::with_capacity(len);
        for i in 0..len {
            let val = self.0[HEIGHT - y - i - 1][x + i];
            buf.push(val);
        }
        buf
    }

    /// Generate the base image from inputs. The result can be displayed as is or
    /// updated via scale, position, or angle then displayed again after `update()`.
    #[inline]
    pub fn from_png(
        path: &Path,
        duration: Option<Duration>,
        bright: f32,
    ) -> Result<Self, AnimeError> {
        use pix::el::Pixel;
        let data = std::fs::read(path)?;
        let data = std::io::Cursor::new(data);
        let decoder = png_pong::Decoder::new(data)?.into_steps();
        let png_pong::Step { raster, delay: _ } = decoder.last().ok_or(AnimeError::NoFrames)??;

        let mut matrix = AnimeDiagonal::new(duration);

        let width;
        match raster {
            png_pong::PngRaster::Graya8(ras) => {
                width = ras.width();
                for (y, row) in ras.pixels().chunks(width as usize).enumerate() {
                    for (x, px) in row.iter().enumerate() {
                        let v = <u8>::from(px.one() * bright);
                        matrix.0[y][x] = v;
                    }
                }
            }
            _ => return Err(AnimeError::Format),
        };

        Ok(matrix)
    }
}

impl From<&AnimeDiagonal> for AnimeDataBuffer {
    /// Do conversion from the nested Vec in AnimeMatrix to the two required
    /// packets suitable for sending over USB
    #[inline]
    fn from(anime: &AnimeDiagonal) -> Self {
        let mut buf = vec![0u8; ANIME_DATA_LEN];

        buf[1..=32].copy_from_slice(&anime.get_row(0, 3, 32));
        buf[34..=66].copy_from_slice(&anime.get_row(0, 2, 33));
        buf[69..=101].copy_from_slice(&anime.get_row(1, 2, 33)); // ?!
        buf[102..=134].copy_from_slice(&anime.get_row(1, 1, 33));
        buf[137..=169].copy_from_slice(&anime.get_row(2, 1, 33));
        buf[170..=202].copy_from_slice(&anime.get_row(2, 0, 33));
        buf[204..=236].copy_from_slice(&anime.get_row(3, 0, 33));
        buf[237..=268].copy_from_slice(&anime.get_row(4, 0, 32));
        buf[270..=301].copy_from_slice(&anime.get_row(5, 0, 32));
        buf[302..=332].copy_from_slice(&anime.get_row(6, 0, 31));
        buf[334..=364].copy_from_slice(&anime.get_row(7, 0, 31));
        buf[365..=394].copy_from_slice(&anime.get_row(8, 0, 30));
        buf[396..=425].copy_from_slice(&anime.get_row(9, 0, 30));
        buf[426..=454].copy_from_slice(&anime.get_row(10, 0, 29));
        buf[456..=484].copy_from_slice(&anime.get_row(11, 0, 29));
        buf[485..=512].copy_from_slice(&anime.get_row(12, 0, 28));
        buf[514..=541].copy_from_slice(&anime.get_row(13, 0, 28));
        buf[542..=568].copy_from_slice(&anime.get_row(14, 0, 27));
        buf[570..=596].copy_from_slice(&anime.get_row(15, 0, 27));
        buf[597..=622].copy_from_slice(&anime.get_row(16, 0, 26));
        buf[624..=649].copy_from_slice(&anime.get_row(17, 0, 26));
        buf[650..=674].copy_from_slice(&anime.get_row(18, 0, 25));
        buf[676..=700].copy_from_slice(&anime.get_row(19, 0, 25));
        buf[701..=724].copy_from_slice(&anime.get_row(20, 0, 24));
        buf[726..=749].copy_from_slice(&anime.get_row(21, 0, 24));
        buf[750..=772].copy_from_slice(&anime.get_row(22, 0, 23));
        buf[774..=796].copy_from_slice(&anime.get_row(23, 0, 23));
        buf[797..=818].copy_from_slice(&anime.get_row(24, 0, 22));
        buf[820..=841].copy_from_slice(&anime.get_row(25, 0, 22));
        buf[842..=862].copy_from_slice(&anime.get_row(26, 0, 21));
        buf[864..=884].copy_from_slice(&anime.get_row(27, 0, 21));
        buf[885..=904].copy_from_slice(&anime.get_row(28, 0, 20));
        buf[906..=925].copy_from_slice(&anime.get_row(29, 0, 20));
        buf[926..=944].copy_from_slice(&anime.get_row(30, 0, 19));
        buf[946..=964].copy_from_slice(&anime.get_row(31, 0, 19));
        buf[965..=982].copy_from_slice(&anime.get_row(32, 0, 18));
        buf[984..=1001].copy_from_slice(&anime.get_row(33, 0, 18));
        buf[1002..=1018].copy_from_slice(&anime.get_row(34, 0, 17));
        buf[1020..=1036].copy_from_slice(&anime.get_row(35, 0, 17));
        buf[1037..=1052].copy_from_slice(&anime.get_row(36, 0, 16));
        buf[1054..=1069].copy_from_slice(&anime.get_row(37, 0, 16));
        buf[1070..=1084].copy_from_slice(&anime.get_row(38, 0, 15));
        buf[1086..=1100].copy_from_slice(&anime.get_row(39, 0, 15));
        buf[1101..=1114].copy_from_slice(&anime.get_row(40, 0, 14));
        buf[1116..=1129].copy_from_slice(&anime.get_row(41, 0, 14));
        buf[1130..=1142].copy_from_slice(&anime.get_row(42, 0, 13));
        buf[1144..=1156].copy_from_slice(&anime.get_row(43, 0, 13));
        buf[1157..=1168].copy_from_slice(&anime.get_row(44, 0, 12));
        buf[1170..=1181].copy_from_slice(&anime.get_row(45, 0, 12));
        buf[1182..=1192].copy_from_slice(&anime.get_row(46, 0, 11));
        buf[1194..=1204].copy_from_slice(&anime.get_row(47, 0, 11));
        buf[1205..=1214].copy_from_slice(&anime.get_row(48, 0, 10));
        buf[1216..=1225].copy_from_slice(&anime.get_row(49, 0, 10));
        buf[1226..=1234].copy_from_slice(&anime.get_row(50, 0, 9));
        buf[1236..=1244].copy_from_slice(&anime.get_row(51, 0, 9));

        AnimeDataBuffer::from_vec(buf)
    }
}
