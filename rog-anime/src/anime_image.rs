use std::path::Path;

pub use glam::Vec2;
use glam::{Mat3, Vec3};

use crate::{
    anime_data::{AniMeDataBuffer, ANIME_DATA_LEN},
    error::AnimeError,
};

const LED_PIXEL_LEN: usize = 1244;

#[derive(Copy, Clone, Debug, Default)]
struct Pixel {
    color: u32,
    alpha: f32,
}

/// A single LED position and brightness. The intention of this struct
/// is to be used to sample an image and set the LED brightness.
///
/// The position of the Led in `LedPositions` determines the placement in the final
/// data packets when written to the AniMe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Led(f32, f32, u8);

impl Led {
    const fn new(x: f32, y: f32) -> Self {
        Led(x, y, 0)
    }

    pub(crate) const fn x(&self) -> f32 {
        self.0
    }

    pub(crate) const fn y(&self) -> f32 {
        self.1
    }

    const fn bright(&self) -> u8 {
        self.2
    }
    
    fn set_bright(&mut self, a: u8) {
        self.2 = a;
    }
}

/// Container of `Led`, each of which specifies a position within the image
/// The main use of this is to position and sample colours for the final image
/// to show on AniMe
pub struct AniMeImage {
    pub scale: Vec2,
    /// Angle in radians
    pub angle: f32,
    pub translation: Vec2,
    /// Brightness of final image, `0.0` = off, `1.0` = full
    pub bright: f32,
    /// Positions of all the LEDs
    led_pos: [Option<Led>; LED_PIXEL_LEN],
    /// THe image data for sampling
    img_pixels: Vec<Pixel>,
    width: u32,
}

impl AniMeImage {
    const fn new(
        scale: Vec2,
        angle: f32,
        translation: Vec2,
        bright: f32,
        pixels: Vec<Pixel>,
        width: u32,
    ) -> Self {
        Self {
            scale,
            angle,
            translation,
            bright,
            led_pos: LED_IMAGE_POSITIONS,
            img_pixels: pixels,
            width,
        }
    }

    /// Scale ratio in CM
    const fn scale_x() -> f32 {
        0.8
    }

    /// Scale ratio in CM
    const fn scale_y() -> f32 {
        0.3
    }

    /// Get the starting X position for the data we actually require when writing
    /// it out to LEDs
    const fn first_x(y: u32) -> u32 {
        if y < 5 {
            return 0;
        }
        (y + 1) / 2 - 3
    }

    /// Width in LED count
    const fn width(y: u32) -> u32 {
        if y < 5 {
            return 33;
        }
        36 - (y + 1) / 2
    }

    fn phys_width() -> f32 {
        (32.0 - -0.5 + 1.0) * Self::scale_x()
    }

    /// Height in LED count
    const fn height() -> u32 {
        55
    }

    fn phys_height() -> f32 {
        (54.0 + 1.0) * Self::scale_y()
    }

    const fn pitch(y: u32) -> u32 {
        match y {
            0 | 2 | 4 => 33,
            1 | 3 => 35,
            _ => 36 - y / 2,
        }
    }

    /// Really only used to generate the output for including as a full const in `LED_IMAGE_POSITIONS`
    pub fn generate() -> Vec<Option<Led>> {
        (0..AniMeImage::height())
            .flat_map(|y| {
                (0..AniMeImage::pitch(y)).map(move |l| {
                    if l < AniMeImage::width(y) {
                        let x = AniMeImage::first_x(y) + l;
                        Some(Led::new(x as f32 - 0.5 * (y % 2) as f32, y as f32))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Called after setting new angle, position, or scale to refresh the image
    /// samples, the result can then been transformed to the appropriate data
    /// for displaying
    pub fn update(&mut self) {
        let width = self.width as i32;
        let height = self.img_pixels.len() as i32 / width;
        let led_from_px = self.put(width as f32, height as f32);
        // Steps should be configurable as "sharpness"
        let du = led_from_px * Vec3::new(-0.5, 0.5, 0.0);
        let dv = led_from_px * Vec3::new(0.5, 0.5, 0.0);

        for led in self.led_pos.iter_mut() {
            if let Some(led) = led {
                let mut sum = 0.0;
                let mut alpha = 0.0;
                let mut count = 0;

                let pos = Vec3::new(led.x(), led.y(), 1.0);
                let x0 = led_from_px.mul_vec3(pos + Vec3::new(0.0, -0.5, 0.0));

                const GROUP: [f32; 4] = [0.0, 0.5, 1.0, 1.5];
                for u in GROUP.iter() {
                    for v in GROUP.iter() {
                        let sample = x0 + *u * du + *v * dv;

                        let mut y = sample.y as i32;
                        if y > height - 1 {
                            y = height - 1
                        } else if y < 0 {
                            y = 0;
                        }

                        let mut x = sample.x as i32;
                        if x > width - 1 {
                            x = width - 1;
                        } else if x < 0 {
                            x = 0;
                        }

                        let p = self.img_pixels[(x + (y * width)) as usize];
                        sum += p.color as f32;
                        alpha += p.alpha;
                        count += 1;
                    }
                }
                alpha /= count as f32;
                sum /= count as f32;
                led.set_bright((sum * self.bright * alpha) as u8);
            }
        }
    }

    fn put(&self, bmp_w: f32, bmp_h: f32) -> Mat3 {
        // Center of image
        let center = Mat3::from_translation(Vec2::new(-0.5 * bmp_w, -0.5 * bmp_h));
        // Find the scale required for cleanly showing the image
        let h = AniMeImage::phys_height() / bmp_h;
        let mut base_scale = AniMeImage::phys_width() / bmp_w;
        if base_scale > h {
            base_scale = h;
        }

        let cm_from_px = Mat3::from_scale(Vec2::new(base_scale, base_scale));

        let led_from_cm = Mat3::from_scale(Vec2::new(
            1.0 / AniMeImage::scale_x(),
            1.0 / AniMeImage::scale_y(),
        ));

        let transform =
            Mat3::from_scale_angle_translation(self.scale, self.angle, self.translation);

        let pos_in_leds = Mat3::from_translation(Vec2::new(20.0, 20.0));
        // Get LED-to-image coords
        let led_from_px = pos_in_leds * led_from_cm * transform * cm_from_px * center;

        led_from_px.inverse()
    }

    /// Generate the base image from inputs. The result can be displayed as is or
    /// updated via scale, position, or angle then displayed again after `update()`.
    pub fn from_png(
        path: &Path,
        scale: Vec2,
        angle: f32,
        translation: Vec2,
        bright: f32,
    ) -> Result<Self, AnimeError> {
        use pix::el::Pixel;
        let data = std::fs::read(path)?;
        let data = std::io::Cursor::new(data);
        let decoder = png_pong::Decoder::new(data)?.into_steps();
        let png_pong::Step { raster, delay: _ } = decoder
            .last()
            .ok_or(AnimeError::NoFrames)??;

        let width;
        let pixels = match raster {
            png_pong::PngRaster::Graya8(ras) => {
                width = ras.width();
                ras.pixels()
                    .iter()
                    .map(|px| crate::anime_image::Pixel {
                        color: <u8>::from(px.one()) as u32,
                        alpha: <f32>::from(px.alpha()),
                    })
                    .collect()
            }
            _ => return Err(AnimeError::Format),
        };

        let mut matrix = AniMeImage::new(scale, angle, translation, bright, pixels, width);

        matrix.update();
        Ok(matrix)
    }
}

impl From<&AniMeImage> for AniMeDataBuffer {
    /// Do conversion from the nested Vec in AniMeMatrix to the two required
    /// packets suitable for sending over USB
    #[inline]
    fn from(leds: &AniMeImage) -> Self {
        let mut l: Vec<u8> = leds
            .led_pos
            .iter()
            .map(|l| if let Some(l) = l { l.bright() } else { 0 })
            .collect();
        let mut v = Vec::with_capacity(ANIME_DATA_LEN);
        v.push(0);
        v.append(&mut l);
        v.append(&mut vec![0u8; 9]);
        AniMeDataBuffer::from_vec(v)
    }
}

/// Data starts at first index which means that when mapping this data to the final
/// USB packet it must start from index 8, not 7.
///
/// Verbatim copy of `generate()`. `LED_IMAGE_POSITIONS` is `const` so prefer this.
pub const LED_IMAGE_POSITIONS: [Option<Led>; LED_PIXEL_LEN] = [
    Some(Led(0.0, 0.0, 0)),
    Some(Led(1.0, 0.0, 0)),
    Some(Led(2.0, 0.0, 0)),
    Some(Led(3.0, 0.0, 0)),
    Some(Led(4.0, 0.0, 0)),
    Some(Led(5.0, 0.0, 0)),
    Some(Led(6.0, 0.0, 0)),
    Some(Led(7.0, 0.0, 0)),
    Some(Led(8.0, 0.0, 0)),
    Some(Led(9.0, 0.0, 0)),
    Some(Led(10.0, 0.0, 0)),
    Some(Led(11.0, 0.0, 0)),
    Some(Led(12.0, 0.0, 0)),
    Some(Led(13.0, 0.0, 0)),
    Some(Led(14.0, 0.0, 0)),
    Some(Led(15.0, 0.0, 0)),
    Some(Led(16.0, 0.0, 0)),
    Some(Led(17.0, 0.0, 0)),
    Some(Led(18.0, 0.0, 0)),
    Some(Led(19.0, 0.0, 0)),
    Some(Led(20.0, 0.0, 0)),
    Some(Led(21.0, 0.0, 0)),
    Some(Led(22.0, 0.0, 0)),
    Some(Led(23.0, 0.0, 0)),
    Some(Led(24.0, 0.0, 0)),
    Some(Led(25.0, 0.0, 0)),
    Some(Led(26.0, 0.0, 0)),
    Some(Led(27.0, 0.0, 0)),
    Some(Led(28.0, 0.0, 0)),
    Some(Led(29.0, 0.0, 0)),
    Some(Led(30.0, 0.0, 0)),
    Some(Led(31.0, 0.0, 0)),
    Some(Led(32.0, 0.0, 0)),
    Some(Led(-0.5, 1.0, 0)),
    Some(Led(0.5, 1.0, 0)),
    Some(Led(1.5, 1.0, 0)),
    Some(Led(2.5, 1.0, 0)),
    Some(Led(3.5, 1.0, 0)),
    Some(Led(4.5, 1.0, 0)),
    Some(Led(5.5, 1.0, 0)),
    Some(Led(6.5, 1.0, 0)),
    Some(Led(7.5, 1.0, 0)),
    Some(Led(8.5, 1.0, 0)),
    Some(Led(9.5, 1.0, 0)),
    Some(Led(10.5, 1.0, 0)),
    Some(Led(11.5, 1.0, 0)),
    Some(Led(12.5, 1.0, 0)),
    Some(Led(13.5, 1.0, 0)),
    Some(Led(14.5, 1.0, 0)),
    Some(Led(15.5, 1.0, 0)),
    Some(Led(16.5, 1.0, 0)),
    Some(Led(17.5, 1.0, 0)),
    Some(Led(18.5, 1.0, 0)),
    Some(Led(19.5, 1.0, 0)),
    Some(Led(20.5, 1.0, 0)),
    Some(Led(21.5, 1.0, 0)),
    Some(Led(22.5, 1.0, 0)),
    Some(Led(23.5, 1.0, 0)),
    Some(Led(24.5, 1.0, 0)),
    Some(Led(25.5, 1.0, 0)),
    Some(Led(26.5, 1.0, 0)),
    Some(Led(27.5, 1.0, 0)),
    Some(Led(28.5, 1.0, 0)),
    Some(Led(29.5, 1.0, 0)),
    Some(Led(30.5, 1.0, 0)),
    Some(Led(31.5, 1.0, 0)),
    None,
    None,
    Some(Led(0.0, 2.0, 0)),
    Some(Led(1.0, 2.0, 0)),
    Some(Led(2.0, 2.0, 0)),
    Some(Led(3.0, 2.0, 0)),
    Some(Led(4.0, 2.0, 0)),
    Some(Led(5.0, 2.0, 0)),
    Some(Led(6.0, 2.0, 0)),
    Some(Led(7.0, 2.0, 0)),
    Some(Led(8.0, 2.0, 0)),
    Some(Led(9.0, 2.0, 0)),
    Some(Led(10.0, 2.0, 0)),
    Some(Led(11.0, 2.0, 0)),
    Some(Led(12.0, 2.0, 0)),
    Some(Led(13.0, 2.0, 0)),
    Some(Led(14.0, 2.0, 0)),
    Some(Led(15.0, 2.0, 0)),
    Some(Led(16.0, 2.0, 0)),
    Some(Led(17.0, 2.0, 0)),
    Some(Led(18.0, 2.0, 0)),
    Some(Led(19.0, 2.0, 0)),
    Some(Led(20.0, 2.0, 0)),
    Some(Led(21.0, 2.0, 0)),
    Some(Led(22.0, 2.0, 0)),
    Some(Led(23.0, 2.0, 0)),
    Some(Led(24.0, 2.0, 0)),
    Some(Led(25.0, 2.0, 0)),
    Some(Led(26.0, 2.0, 0)),
    Some(Led(27.0, 2.0, 0)),
    Some(Led(28.0, 2.0, 0)),
    Some(Led(29.0, 2.0, 0)),
    Some(Led(30.0, 2.0, 0)),
    Some(Led(31.0, 2.0, 0)),
    Some(Led(32.0, 2.0, 0)),
    Some(Led(-0.5, 3.0, 0)),
    Some(Led(0.5, 3.0, 0)),
    Some(Led(1.5, 3.0, 0)),
    Some(Led(2.5, 3.0, 0)),
    Some(Led(3.5, 3.0, 0)),
    Some(Led(4.5, 3.0, 0)),
    Some(Led(5.5, 3.0, 0)),
    Some(Led(6.5, 3.0, 0)),
    Some(Led(7.5, 3.0, 0)),
    Some(Led(8.5, 3.0, 0)),
    Some(Led(9.5, 3.0, 0)),
    Some(Led(10.5, 3.0, 0)),
    Some(Led(11.5, 3.0, 0)),
    Some(Led(12.5, 3.0, 0)),
    Some(Led(13.5, 3.0, 0)),
    Some(Led(14.5, 3.0, 0)),
    Some(Led(15.5, 3.0, 0)),
    Some(Led(16.5, 3.0, 0)),
    Some(Led(17.5, 3.0, 0)),
    Some(Led(18.5, 3.0, 0)),
    Some(Led(19.5, 3.0, 0)),
    Some(Led(20.5, 3.0, 0)),
    Some(Led(21.5, 3.0, 0)),
    Some(Led(22.5, 3.0, 0)),
    Some(Led(23.5, 3.0, 0)),
    Some(Led(24.5, 3.0, 0)),
    Some(Led(25.5, 3.0, 0)),
    Some(Led(26.5, 3.0, 0)),
    Some(Led(27.5, 3.0, 0)),
    Some(Led(28.5, 3.0, 0)),
    Some(Led(29.5, 3.0, 0)),
    Some(Led(30.5, 3.0, 0)),
    Some(Led(31.5, 3.0, 0)),
    None,
    None,
    Some(Led(0.0, 4.0, 0)),
    Some(Led(1.0, 4.0, 0)),
    Some(Led(2.0, 4.0, 0)),
    Some(Led(3.0, 4.0, 0)),
    Some(Led(4.0, 4.0, 0)),
    Some(Led(5.0, 4.0, 0)),
    Some(Led(6.0, 4.0, 0)),
    Some(Led(7.0, 4.0, 0)),
    Some(Led(8.0, 4.0, 0)),
    Some(Led(9.0, 4.0, 0)),
    Some(Led(10.0, 4.0, 0)),
    Some(Led(11.0, 4.0, 0)),
    Some(Led(12.0, 4.0, 0)),
    Some(Led(13.0, 4.0, 0)),
    Some(Led(14.0, 4.0, 0)),
    Some(Led(15.0, 4.0, 0)),
    Some(Led(16.0, 4.0, 0)),
    Some(Led(17.0, 4.0, 0)),
    Some(Led(18.0, 4.0, 0)),
    Some(Led(19.0, 4.0, 0)),
    Some(Led(20.0, 4.0, 0)),
    Some(Led(21.0, 4.0, 0)),
    Some(Led(22.0, 4.0, 0)),
    Some(Led(23.0, 4.0, 0)),
    Some(Led(24.0, 4.0, 0)),
    Some(Led(25.0, 4.0, 0)),
    Some(Led(26.0, 4.0, 0)),
    Some(Led(27.0, 4.0, 0)),
    Some(Led(28.0, 4.0, 0)),
    Some(Led(29.0, 4.0, 0)),
    Some(Led(30.0, 4.0, 0)),
    Some(Led(31.0, 4.0, 0)),
    Some(Led(32.0, 4.0, 0)),
    Some(Led(-0.5, 5.0, 0)),
    Some(Led(0.5, 5.0, 0)),
    Some(Led(1.5, 5.0, 0)),
    Some(Led(2.5, 5.0, 0)),
    Some(Led(3.5, 5.0, 0)),
    Some(Led(4.5, 5.0, 0)),
    Some(Led(5.5, 5.0, 0)),
    Some(Led(6.5, 5.0, 0)),
    Some(Led(7.5, 5.0, 0)),
    Some(Led(8.5, 5.0, 0)),
    Some(Led(9.5, 5.0, 0)),
    Some(Led(10.5, 5.0, 0)),
    Some(Led(11.5, 5.0, 0)),
    Some(Led(12.5, 5.0, 0)),
    Some(Led(13.5, 5.0, 0)),
    Some(Led(14.5, 5.0, 0)),
    Some(Led(15.5, 5.0, 0)),
    Some(Led(16.5, 5.0, 0)),
    Some(Led(17.5, 5.0, 0)),
    Some(Led(18.5, 5.0, 0)),
    Some(Led(19.5, 5.0, 0)),
    Some(Led(20.5, 5.0, 0)),
    Some(Led(21.5, 5.0, 0)),
    Some(Led(22.5, 5.0, 0)),
    Some(Led(23.5, 5.0, 0)),
    Some(Led(24.5, 5.0, 0)),
    Some(Led(25.5, 5.0, 0)),
    Some(Led(26.5, 5.0, 0)),
    Some(Led(27.5, 5.0, 0)),
    Some(Led(28.5, 5.0, 0)),
    Some(Led(29.5, 5.0, 0)),
    Some(Led(30.5, 5.0, 0)),
    Some(Led(31.5, 5.0, 0)),
    None,
    Some(Led(0.0, 6.0, 0)),
    Some(Led(1.0, 6.0, 0)),
    Some(Led(2.0, 6.0, 0)),
    Some(Led(3.0, 6.0, 0)),
    Some(Led(4.0, 6.0, 0)),
    Some(Led(5.0, 6.0, 0)),
    Some(Led(6.0, 6.0, 0)),
    Some(Led(7.0, 6.0, 0)),
    Some(Led(8.0, 6.0, 0)),
    Some(Led(9.0, 6.0, 0)),
    Some(Led(10.0, 6.0, 0)),
    Some(Led(11.0, 6.0, 0)),
    Some(Led(12.0, 6.0, 0)),
    Some(Led(13.0, 6.0, 0)),
    Some(Led(14.0, 6.0, 0)),
    Some(Led(15.0, 6.0, 0)),
    Some(Led(16.0, 6.0, 0)),
    Some(Led(17.0, 6.0, 0)),
    Some(Led(18.0, 6.0, 0)),
    Some(Led(19.0, 6.0, 0)),
    Some(Led(20.0, 6.0, 0)),
    Some(Led(21.0, 6.0, 0)),
    Some(Led(22.0, 6.0, 0)),
    Some(Led(23.0, 6.0, 0)),
    Some(Led(24.0, 6.0, 0)),
    Some(Led(25.0, 6.0, 0)),
    Some(Led(26.0, 6.0, 0)),
    Some(Led(27.0, 6.0, 0)),
    Some(Led(28.0, 6.0, 0)),
    Some(Led(29.0, 6.0, 0)),
    Some(Led(30.0, 6.0, 0)),
    Some(Led(31.0, 6.0, 0)),
    Some(Led(32.0, 6.0, 0)),
    Some(Led(0.5, 7.0, 0)),
    Some(Led(1.5, 7.0, 0)),
    Some(Led(2.5, 7.0, 0)),
    Some(Led(3.5, 7.0, 0)),
    Some(Led(4.5, 7.0, 0)),
    Some(Led(5.5, 7.0, 0)),
    Some(Led(6.5, 7.0, 0)),
    Some(Led(7.5, 7.0, 0)),
    Some(Led(8.5, 7.0, 0)),
    Some(Led(9.5, 7.0, 0)),
    Some(Led(10.5, 7.0, 0)),
    Some(Led(11.5, 7.0, 0)),
    Some(Led(12.5, 7.0, 0)),
    Some(Led(13.5, 7.0, 0)),
    Some(Led(14.5, 7.0, 0)),
    Some(Led(15.5, 7.0, 0)),
    Some(Led(16.5, 7.0, 0)),
    Some(Led(17.5, 7.0, 0)),
    Some(Led(18.5, 7.0, 0)),
    Some(Led(19.5, 7.0, 0)),
    Some(Led(20.5, 7.0, 0)),
    Some(Led(21.5, 7.0, 0)),
    Some(Led(22.5, 7.0, 0)),
    Some(Led(23.5, 7.0, 0)),
    Some(Led(24.5, 7.0, 0)),
    Some(Led(25.5, 7.0, 0)),
    Some(Led(26.5, 7.0, 0)),
    Some(Led(27.5, 7.0, 0)),
    Some(Led(28.5, 7.0, 0)),
    Some(Led(29.5, 7.0, 0)),
    Some(Led(30.5, 7.0, 0)),
    Some(Led(31.5, 7.0, 0)),
    None,
    Some(Led(1.0, 8.0, 0)),
    Some(Led(2.0, 8.0, 0)),
    Some(Led(3.0, 8.0, 0)),
    Some(Led(4.0, 8.0, 0)),
    Some(Led(5.0, 8.0, 0)),
    Some(Led(6.0, 8.0, 0)),
    Some(Led(7.0, 8.0, 0)),
    Some(Led(8.0, 8.0, 0)),
    Some(Led(9.0, 8.0, 0)),
    Some(Led(10.0, 8.0, 0)),
    Some(Led(11.0, 8.0, 0)),
    Some(Led(12.0, 8.0, 0)),
    Some(Led(13.0, 8.0, 0)),
    Some(Led(14.0, 8.0, 0)),
    Some(Led(15.0, 8.0, 0)),
    Some(Led(16.0, 8.0, 0)),
    Some(Led(17.0, 8.0, 0)),
    Some(Led(18.0, 8.0, 0)),
    Some(Led(19.0, 8.0, 0)),
    Some(Led(20.0, 8.0, 0)),
    Some(Led(21.0, 8.0, 0)),
    Some(Led(22.0, 8.0, 0)),
    Some(Led(23.0, 8.0, 0)),
    Some(Led(24.0, 8.0, 0)),
    Some(Led(25.0, 8.0, 0)),
    Some(Led(26.0, 8.0, 0)),
    Some(Led(27.0, 8.0, 0)),
    Some(Led(28.0, 8.0, 0)),
    Some(Led(29.0, 8.0, 0)),
    Some(Led(30.0, 8.0, 0)),
    Some(Led(31.0, 8.0, 0)),
    Some(Led(32.0, 8.0, 0)),
    Some(Led(1.5, 9.0, 0)),
    Some(Led(2.5, 9.0, 0)),
    Some(Led(3.5, 9.0, 0)),
    Some(Led(4.5, 9.0, 0)),
    Some(Led(5.5, 9.0, 0)),
    Some(Led(6.5, 9.0, 0)),
    Some(Led(7.5, 9.0, 0)),
    Some(Led(8.5, 9.0, 0)),
    Some(Led(9.5, 9.0, 0)),
    Some(Led(10.5, 9.0, 0)),
    Some(Led(11.5, 9.0, 0)),
    Some(Led(12.5, 9.0, 0)),
    Some(Led(13.5, 9.0, 0)),
    Some(Led(14.5, 9.0, 0)),
    Some(Led(15.5, 9.0, 0)),
    Some(Led(16.5, 9.0, 0)),
    Some(Led(17.5, 9.0, 0)),
    Some(Led(18.5, 9.0, 0)),
    Some(Led(19.5, 9.0, 0)),
    Some(Led(20.5, 9.0, 0)),
    Some(Led(21.5, 9.0, 0)),
    Some(Led(22.5, 9.0, 0)),
    Some(Led(23.5, 9.0, 0)),
    Some(Led(24.5, 9.0, 0)),
    Some(Led(25.5, 9.0, 0)),
    Some(Led(26.5, 9.0, 0)),
    Some(Led(27.5, 9.0, 0)),
    Some(Led(28.5, 9.0, 0)),
    Some(Led(29.5, 9.0, 0)),
    Some(Led(30.5, 9.0, 0)),
    Some(Led(31.5, 9.0, 0)),
    None,
    Some(Led(2.0, 10.0, 0)),
    Some(Led(3.0, 10.0, 0)),
    Some(Led(4.0, 10.0, 0)),
    Some(Led(5.0, 10.0, 0)),
    Some(Led(6.0, 10.0, 0)),
    Some(Led(7.0, 10.0, 0)),
    Some(Led(8.0, 10.0, 0)),
    Some(Led(9.0, 10.0, 0)),
    Some(Led(10.0, 10.0, 0)),
    Some(Led(11.0, 10.0, 0)),
    Some(Led(12.0, 10.0, 0)),
    Some(Led(13.0, 10.0, 0)),
    Some(Led(14.0, 10.0, 0)),
    Some(Led(15.0, 10.0, 0)),
    Some(Led(16.0, 10.0, 0)),
    Some(Led(17.0, 10.0, 0)),
    Some(Led(18.0, 10.0, 0)),
    Some(Led(19.0, 10.0, 0)),
    Some(Led(20.0, 10.0, 0)),
    Some(Led(21.0, 10.0, 0)),
    Some(Led(22.0, 10.0, 0)),
    Some(Led(23.0, 10.0, 0)),
    Some(Led(24.0, 10.0, 0)),
    Some(Led(25.0, 10.0, 0)),
    Some(Led(26.0, 10.0, 0)),
    Some(Led(27.0, 10.0, 0)),
    Some(Led(28.0, 10.0, 0)),
    Some(Led(29.0, 10.0, 0)),
    Some(Led(30.0, 10.0, 0)),
    Some(Led(31.0, 10.0, 0)),
    Some(Led(32.0, 10.0, 0)),
    Some(Led(2.5, 11.0, 0)),
    Some(Led(3.5, 11.0, 0)),
    Some(Led(4.5, 11.0, 0)),
    Some(Led(5.5, 11.0, 0)),
    Some(Led(6.5, 11.0, 0)),
    Some(Led(7.5, 11.0, 0)),
    Some(Led(8.5, 11.0, 0)),
    Some(Led(9.5, 11.0, 0)),
    Some(Led(10.5, 11.0, 0)),
    Some(Led(11.5, 11.0, 0)),
    Some(Led(12.5, 11.0, 0)),
    Some(Led(13.5, 11.0, 0)),
    Some(Led(14.5, 11.0, 0)),
    Some(Led(15.5, 11.0, 0)),
    Some(Led(16.5, 11.0, 0)),
    Some(Led(17.5, 11.0, 0)),
    Some(Led(18.5, 11.0, 0)),
    Some(Led(19.5, 11.0, 0)),
    Some(Led(20.5, 11.0, 0)),
    Some(Led(21.5, 11.0, 0)),
    Some(Led(22.5, 11.0, 0)),
    Some(Led(23.5, 11.0, 0)),
    Some(Led(24.5, 11.0, 0)),
    Some(Led(25.5, 11.0, 0)),
    Some(Led(26.5, 11.0, 0)),
    Some(Led(27.5, 11.0, 0)),
    Some(Led(28.5, 11.0, 0)),
    Some(Led(29.5, 11.0, 0)),
    Some(Led(30.5, 11.0, 0)),
    Some(Led(31.5, 11.0, 0)),
    None,
    Some(Led(3.0, 12.0, 0)),
    Some(Led(4.0, 12.0, 0)),
    Some(Led(5.0, 12.0, 0)),
    Some(Led(6.0, 12.0, 0)),
    Some(Led(7.0, 12.0, 0)),
    Some(Led(8.0, 12.0, 0)),
    Some(Led(9.0, 12.0, 0)),
    Some(Led(10.0, 12.0, 0)),
    Some(Led(11.0, 12.0, 0)),
    Some(Led(12.0, 12.0, 0)),
    Some(Led(13.0, 12.0, 0)),
    Some(Led(14.0, 12.0, 0)),
    Some(Led(15.0, 12.0, 0)),
    Some(Led(16.0, 12.0, 0)),
    Some(Led(17.0, 12.0, 0)),
    Some(Led(18.0, 12.0, 0)),
    Some(Led(19.0, 12.0, 0)),
    Some(Led(20.0, 12.0, 0)),
    Some(Led(21.0, 12.0, 0)),
    Some(Led(22.0, 12.0, 0)),
    Some(Led(23.0, 12.0, 0)),
    Some(Led(24.0, 12.0, 0)),
    Some(Led(25.0, 12.0, 0)),
    Some(Led(26.0, 12.0, 0)),
    Some(Led(27.0, 12.0, 0)),
    Some(Led(28.0, 12.0, 0)),
    Some(Led(29.0, 12.0, 0)),
    Some(Led(30.0, 12.0, 0)),
    Some(Led(31.0, 12.0, 0)),
    Some(Led(32.0, 12.0, 0)),
    Some(Led(3.5, 13.0, 0)),
    Some(Led(4.5, 13.0, 0)),
    Some(Led(5.5, 13.0, 0)),
    Some(Led(6.5, 13.0, 0)),
    Some(Led(7.5, 13.0, 0)),
    Some(Led(8.5, 13.0, 0)),
    Some(Led(9.5, 13.0, 0)),
    Some(Led(10.5, 13.0, 0)),
    Some(Led(11.5, 13.0, 0)),
    Some(Led(12.5, 13.0, 0)),
    Some(Led(13.5, 13.0, 0)),
    Some(Led(14.5, 13.0, 0)),
    Some(Led(15.5, 13.0, 0)),
    Some(Led(16.5, 13.0, 0)),
    Some(Led(17.5, 13.0, 0)),
    Some(Led(18.5, 13.0, 0)),
    Some(Led(19.5, 13.0, 0)),
    Some(Led(20.5, 13.0, 0)),
    Some(Led(21.5, 13.0, 0)),
    Some(Led(22.5, 13.0, 0)),
    Some(Led(23.5, 13.0, 0)),
    Some(Led(24.5, 13.0, 0)),
    Some(Led(25.5, 13.0, 0)),
    Some(Led(26.5, 13.0, 0)),
    Some(Led(27.5, 13.0, 0)),
    Some(Led(28.5, 13.0, 0)),
    Some(Led(29.5, 13.0, 0)),
    Some(Led(30.5, 13.0, 0)),
    Some(Led(31.5, 13.0, 0)),
    None,
    Some(Led(4.0, 14.0, 0)),
    Some(Led(5.0, 14.0, 0)),
    Some(Led(6.0, 14.0, 0)),
    Some(Led(7.0, 14.0, 0)),
    Some(Led(8.0, 14.0, 0)),
    Some(Led(9.0, 14.0, 0)),
    Some(Led(10.0, 14.0, 0)),
    Some(Led(11.0, 14.0, 0)),
    Some(Led(12.0, 14.0, 0)),
    Some(Led(13.0, 14.0, 0)),
    Some(Led(14.0, 14.0, 0)),
    Some(Led(15.0, 14.0, 0)),
    Some(Led(16.0, 14.0, 0)),
    Some(Led(17.0, 14.0, 0)),
    Some(Led(18.0, 14.0, 0)),
    Some(Led(19.0, 14.0, 0)),
    Some(Led(20.0, 14.0, 0)),
    Some(Led(21.0, 14.0, 0)),
    Some(Led(22.0, 14.0, 0)),
    Some(Led(23.0, 14.0, 0)),
    Some(Led(24.0, 14.0, 0)),
    Some(Led(25.0, 14.0, 0)),
    Some(Led(26.0, 14.0, 0)),
    Some(Led(27.0, 14.0, 0)),
    Some(Led(28.0, 14.0, 0)),
    Some(Led(29.0, 14.0, 0)),
    Some(Led(30.0, 14.0, 0)),
    Some(Led(31.0, 14.0, 0)),
    Some(Led(32.0, 14.0, 0)),
    Some(Led(4.5, 15.0, 0)),
    Some(Led(5.5, 15.0, 0)),
    Some(Led(6.5, 15.0, 0)),
    Some(Led(7.5, 15.0, 0)),
    Some(Led(8.5, 15.0, 0)),
    Some(Led(9.5, 15.0, 0)),
    Some(Led(10.5, 15.0, 0)),
    Some(Led(11.5, 15.0, 0)),
    Some(Led(12.5, 15.0, 0)),
    Some(Led(13.5, 15.0, 0)),
    Some(Led(14.5, 15.0, 0)),
    Some(Led(15.5, 15.0, 0)),
    Some(Led(16.5, 15.0, 0)),
    Some(Led(17.5, 15.0, 0)),
    Some(Led(18.5, 15.0, 0)),
    Some(Led(19.5, 15.0, 0)),
    Some(Led(20.5, 15.0, 0)),
    Some(Led(21.5, 15.0, 0)),
    Some(Led(22.5, 15.0, 0)),
    Some(Led(23.5, 15.0, 0)),
    Some(Led(24.5, 15.0, 0)),
    Some(Led(25.5, 15.0, 0)),
    Some(Led(26.5, 15.0, 0)),
    Some(Led(27.5, 15.0, 0)),
    Some(Led(28.5, 15.0, 0)),
    Some(Led(29.5, 15.0, 0)),
    Some(Led(30.5, 15.0, 0)),
    Some(Led(31.5, 15.0, 0)),
    None,
    Some(Led(5.0, 16.0, 0)),
    Some(Led(6.0, 16.0, 0)),
    Some(Led(7.0, 16.0, 0)),
    Some(Led(8.0, 16.0, 0)),
    Some(Led(9.0, 16.0, 0)),
    Some(Led(10.0, 16.0, 0)),
    Some(Led(11.0, 16.0, 0)),
    Some(Led(12.0, 16.0, 0)),
    Some(Led(13.0, 16.0, 0)),
    Some(Led(14.0, 16.0, 0)),
    Some(Led(15.0, 16.0, 0)),
    Some(Led(16.0, 16.0, 0)),
    Some(Led(17.0, 16.0, 0)),
    Some(Led(18.0, 16.0, 0)),
    Some(Led(19.0, 16.0, 0)),
    Some(Led(20.0, 16.0, 0)),
    Some(Led(21.0, 16.0, 0)),
    Some(Led(22.0, 16.0, 0)),
    Some(Led(23.0, 16.0, 0)),
    Some(Led(24.0, 16.0, 0)),
    Some(Led(25.0, 16.0, 0)),
    Some(Led(26.0, 16.0, 0)),
    Some(Led(27.0, 16.0, 0)),
    Some(Led(28.0, 16.0, 0)),
    Some(Led(29.0, 16.0, 0)),
    Some(Led(30.0, 16.0, 0)),
    Some(Led(31.0, 16.0, 0)),
    Some(Led(32.0, 16.0, 0)),
    Some(Led(5.5, 17.0, 0)),
    Some(Led(6.5, 17.0, 0)),
    Some(Led(7.5, 17.0, 0)),
    Some(Led(8.5, 17.0, 0)),
    Some(Led(9.5, 17.0, 0)),
    Some(Led(10.5, 17.0, 0)),
    Some(Led(11.5, 17.0, 0)),
    Some(Led(12.5, 17.0, 0)),
    Some(Led(13.5, 17.0, 0)),
    Some(Led(14.5, 17.0, 0)),
    Some(Led(15.5, 17.0, 0)),
    Some(Led(16.5, 17.0, 0)),
    Some(Led(17.5, 17.0, 0)),
    Some(Led(18.5, 17.0, 0)),
    Some(Led(19.5, 17.0, 0)),
    Some(Led(20.5, 17.0, 0)),
    Some(Led(21.5, 17.0, 0)),
    Some(Led(22.5, 17.0, 0)),
    Some(Led(23.5, 17.0, 0)),
    Some(Led(24.5, 17.0, 0)),
    Some(Led(25.5, 17.0, 0)),
    Some(Led(26.5, 17.0, 0)),
    Some(Led(27.5, 17.0, 0)),
    Some(Led(28.5, 17.0, 0)),
    Some(Led(29.5, 17.0, 0)),
    Some(Led(30.5, 17.0, 0)),
    Some(Led(31.5, 17.0, 0)),
    None,
    Some(Led(6.0, 18.0, 0)),
    Some(Led(7.0, 18.0, 0)),
    Some(Led(8.0, 18.0, 0)),
    Some(Led(9.0, 18.0, 0)),
    Some(Led(10.0, 18.0, 0)),
    Some(Led(11.0, 18.0, 0)),
    Some(Led(12.0, 18.0, 0)),
    Some(Led(13.0, 18.0, 0)),
    Some(Led(14.0, 18.0, 0)),
    Some(Led(15.0, 18.0, 0)),
    Some(Led(16.0, 18.0, 0)),
    Some(Led(17.0, 18.0, 0)),
    Some(Led(18.0, 18.0, 0)),
    Some(Led(19.0, 18.0, 0)),
    Some(Led(20.0, 18.0, 0)),
    Some(Led(21.0, 18.0, 0)),
    Some(Led(22.0, 18.0, 0)),
    Some(Led(23.0, 18.0, 0)),
    Some(Led(24.0, 18.0, 0)),
    Some(Led(25.0, 18.0, 0)),
    Some(Led(26.0, 18.0, 0)),
    Some(Led(27.0, 18.0, 0)),
    Some(Led(28.0, 18.0, 0)),
    Some(Led(29.0, 18.0, 0)),
    Some(Led(30.0, 18.0, 0)),
    Some(Led(31.0, 18.0, 0)),
    Some(Led(32.0, 18.0, 0)),
    Some(Led(6.5, 19.0, 0)),
    Some(Led(7.5, 19.0, 0)),
    Some(Led(8.5, 19.0, 0)),
    Some(Led(9.5, 19.0, 0)),
    Some(Led(10.5, 19.0, 0)),
    Some(Led(11.5, 19.0, 0)),
    Some(Led(12.5, 19.0, 0)),
    Some(Led(13.5, 19.0, 0)),
    Some(Led(14.5, 19.0, 0)),
    Some(Led(15.5, 19.0, 0)),
    Some(Led(16.5, 19.0, 0)),
    Some(Led(17.5, 19.0, 0)),
    Some(Led(18.5, 19.0, 0)),
    Some(Led(19.5, 19.0, 0)),
    Some(Led(20.5, 19.0, 0)),
    Some(Led(21.5, 19.0, 0)),
    Some(Led(22.5, 19.0, 0)),
    Some(Led(23.5, 19.0, 0)),
    Some(Led(24.5, 19.0, 0)),
    Some(Led(25.5, 19.0, 0)),
    Some(Led(26.5, 19.0, 0)),
    Some(Led(27.5, 19.0, 0)),
    Some(Led(28.5, 19.0, 0)),
    Some(Led(29.5, 19.0, 0)),
    Some(Led(30.5, 19.0, 0)),
    Some(Led(31.5, 19.0, 0)),
    None,
    Some(Led(7.0, 20.0, 0)),
    Some(Led(8.0, 20.0, 0)),
    Some(Led(9.0, 20.0, 0)),
    Some(Led(10.0, 20.0, 0)),
    Some(Led(11.0, 20.0, 0)),
    Some(Led(12.0, 20.0, 0)),
    Some(Led(13.0, 20.0, 0)),
    Some(Led(14.0, 20.0, 0)),
    Some(Led(15.0, 20.0, 0)),
    Some(Led(16.0, 20.0, 0)),
    Some(Led(17.0, 20.0, 0)),
    Some(Led(18.0, 20.0, 0)),
    Some(Led(19.0, 20.0, 0)),
    Some(Led(20.0, 20.0, 0)),
    Some(Led(21.0, 20.0, 0)),
    Some(Led(22.0, 20.0, 0)),
    Some(Led(23.0, 20.0, 0)),
    Some(Led(24.0, 20.0, 0)),
    Some(Led(25.0, 20.0, 0)),
    Some(Led(26.0, 20.0, 0)),
    Some(Led(27.0, 20.0, 0)),
    Some(Led(28.0, 20.0, 0)),
    Some(Led(29.0, 20.0, 0)),
    Some(Led(30.0, 20.0, 0)),
    Some(Led(31.0, 20.0, 0)),
    Some(Led(32.0, 20.0, 0)),
    Some(Led(7.5, 21.0, 0)),
    Some(Led(8.5, 21.0, 0)),
    Some(Led(9.5, 21.0, 0)),
    Some(Led(10.5, 21.0, 0)),
    Some(Led(11.5, 21.0, 0)),
    Some(Led(12.5, 21.0, 0)),
    Some(Led(13.5, 21.0, 0)),
    Some(Led(14.5, 21.0, 0)),
    Some(Led(15.5, 21.0, 0)),
    Some(Led(16.5, 21.0, 0)),
    Some(Led(17.5, 21.0, 0)),
    Some(Led(18.5, 21.0, 0)),
    Some(Led(19.5, 21.0, 0)),
    Some(Led(20.5, 21.0, 0)),
    Some(Led(21.5, 21.0, 0)),
    Some(Led(22.5, 21.0, 0)),
    Some(Led(23.5, 21.0, 0)),
    Some(Led(24.5, 21.0, 0)),
    Some(Led(25.5, 21.0, 0)),
    Some(Led(26.5, 21.0, 0)),
    Some(Led(27.5, 21.0, 0)),
    Some(Led(28.5, 21.0, 0)),
    Some(Led(29.5, 21.0, 0)),
    Some(Led(30.5, 21.0, 0)),
    Some(Led(31.5, 21.0, 0)),
    None,
    Some(Led(8.0, 22.0, 0)),
    Some(Led(9.0, 22.0, 0)),
    Some(Led(10.0, 22.0, 0)),
    Some(Led(11.0, 22.0, 0)),
    Some(Led(12.0, 22.0, 0)),
    Some(Led(13.0, 22.0, 0)),
    Some(Led(14.0, 22.0, 0)),
    Some(Led(15.0, 22.0, 0)),
    Some(Led(16.0, 22.0, 0)),
    Some(Led(17.0, 22.0, 0)),
    Some(Led(18.0, 22.0, 0)),
    Some(Led(19.0, 22.0, 0)),
    Some(Led(20.0, 22.0, 0)),
    Some(Led(21.0, 22.0, 0)),
    Some(Led(22.0, 22.0, 0)),
    Some(Led(23.0, 22.0, 0)),
    Some(Led(24.0, 22.0, 0)),
    Some(Led(25.0, 22.0, 0)),
    Some(Led(26.0, 22.0, 0)),
    Some(Led(27.0, 22.0, 0)),
    Some(Led(28.0, 22.0, 0)),
    Some(Led(29.0, 22.0, 0)),
    Some(Led(30.0, 22.0, 0)),
    Some(Led(31.0, 22.0, 0)),
    Some(Led(32.0, 22.0, 0)),
    Some(Led(8.5, 23.0, 0)),
    Some(Led(9.5, 23.0, 0)),
    Some(Led(10.5, 23.0, 0)),
    Some(Led(11.5, 23.0, 0)),
    Some(Led(12.5, 23.0, 0)),
    Some(Led(13.5, 23.0, 0)),
    Some(Led(14.5, 23.0, 0)),
    Some(Led(15.5, 23.0, 0)),
    Some(Led(16.5, 23.0, 0)),
    Some(Led(17.5, 23.0, 0)),
    Some(Led(18.5, 23.0, 0)),
    Some(Led(19.5, 23.0, 0)),
    Some(Led(20.5, 23.0, 0)),
    Some(Led(21.5, 23.0, 0)),
    Some(Led(22.5, 23.0, 0)),
    Some(Led(23.5, 23.0, 0)),
    Some(Led(24.5, 23.0, 0)),
    Some(Led(25.5, 23.0, 0)),
    Some(Led(26.5, 23.0, 0)),
    Some(Led(27.5, 23.0, 0)),
    Some(Led(28.5, 23.0, 0)),
    Some(Led(29.5, 23.0, 0)),
    Some(Led(30.5, 23.0, 0)),
    Some(Led(31.5, 23.0, 0)),
    None,
    Some(Led(9.0, 24.0, 0)),
    Some(Led(10.0, 24.0, 0)),
    Some(Led(11.0, 24.0, 0)),
    Some(Led(12.0, 24.0, 0)),
    Some(Led(13.0, 24.0, 0)),
    Some(Led(14.0, 24.0, 0)),
    Some(Led(15.0, 24.0, 0)),
    Some(Led(16.0, 24.0, 0)),
    Some(Led(17.0, 24.0, 0)),
    Some(Led(18.0, 24.0, 0)),
    Some(Led(19.0, 24.0, 0)),
    Some(Led(20.0, 24.0, 0)),
    Some(Led(21.0, 24.0, 0)),
    Some(Led(22.0, 24.0, 0)),
    Some(Led(23.0, 24.0, 0)),
    Some(Led(24.0, 24.0, 0)),
    Some(Led(25.0, 24.0, 0)),
    Some(Led(26.0, 24.0, 0)),
    Some(Led(27.0, 24.0, 0)),
    Some(Led(28.0, 24.0, 0)),
    Some(Led(29.0, 24.0, 0)),
    Some(Led(30.0, 24.0, 0)),
    Some(Led(31.0, 24.0, 0)),
    Some(Led(32.0, 24.0, 0)),
    Some(Led(9.5, 25.0, 0)),
    Some(Led(10.5, 25.0, 0)),
    Some(Led(11.5, 25.0, 0)),
    Some(Led(12.5, 25.0, 0)),
    Some(Led(13.5, 25.0, 0)),
    Some(Led(14.5, 25.0, 0)),
    Some(Led(15.5, 25.0, 0)),
    Some(Led(16.5, 25.0, 0)),
    Some(Led(17.5, 25.0, 0)),
    Some(Led(18.5, 25.0, 0)),
    Some(Led(19.5, 25.0, 0)),
    Some(Led(20.5, 25.0, 0)),
    Some(Led(21.5, 25.0, 0)),
    Some(Led(22.5, 25.0, 0)),
    Some(Led(23.5, 25.0, 0)),
    Some(Led(24.5, 25.0, 0)),
    Some(Led(25.5, 25.0, 0)),
    Some(Led(26.5, 25.0, 0)),
    Some(Led(27.5, 25.0, 0)),
    Some(Led(28.5, 25.0, 0)),
    Some(Led(29.5, 25.0, 0)),
    Some(Led(30.5, 25.0, 0)),
    Some(Led(31.5, 25.0, 0)),
    None,
    Some(Led(10.0, 26.0, 0)),
    Some(Led(11.0, 26.0, 0)),
    Some(Led(12.0, 26.0, 0)),
    Some(Led(13.0, 26.0, 0)),
    Some(Led(14.0, 26.0, 0)),
    Some(Led(15.0, 26.0, 0)),
    Some(Led(16.0, 26.0, 0)),
    Some(Led(17.0, 26.0, 0)),
    Some(Led(18.0, 26.0, 0)),
    Some(Led(19.0, 26.0, 0)),
    Some(Led(20.0, 26.0, 0)),
    Some(Led(21.0, 26.0, 0)),
    Some(Led(22.0, 26.0, 0)),
    Some(Led(23.0, 26.0, 0)),
    Some(Led(24.0, 26.0, 0)),
    Some(Led(25.0, 26.0, 0)),
    Some(Led(26.0, 26.0, 0)),
    Some(Led(27.0, 26.0, 0)),
    Some(Led(28.0, 26.0, 0)),
    Some(Led(29.0, 26.0, 0)),
    Some(Led(30.0, 26.0, 0)),
    Some(Led(31.0, 26.0, 0)),
    Some(Led(32.0, 26.0, 0)),
    Some(Led(10.5, 27.0, 0)),
    Some(Led(11.5, 27.0, 0)),
    Some(Led(12.5, 27.0, 0)),
    Some(Led(13.5, 27.0, 0)),
    Some(Led(14.5, 27.0, 0)),
    Some(Led(15.5, 27.0, 0)),
    Some(Led(16.5, 27.0, 0)),
    Some(Led(17.5, 27.0, 0)),
    Some(Led(18.5, 27.0, 0)),
    Some(Led(19.5, 27.0, 0)),
    Some(Led(20.5, 27.0, 0)),
    Some(Led(21.5, 27.0, 0)),
    Some(Led(22.5, 27.0, 0)),
    Some(Led(23.5, 27.0, 0)),
    Some(Led(24.5, 27.0, 0)),
    Some(Led(25.5, 27.0, 0)),
    Some(Led(26.5, 27.0, 0)),
    Some(Led(27.5, 27.0, 0)),
    Some(Led(28.5, 27.0, 0)),
    Some(Led(29.5, 27.0, 0)),
    Some(Led(30.5, 27.0, 0)),
    Some(Led(31.5, 27.0, 0)),
    None,
    Some(Led(11.0, 28.0, 0)),
    Some(Led(12.0, 28.0, 0)),
    Some(Led(13.0, 28.0, 0)),
    Some(Led(14.0, 28.0, 0)),
    Some(Led(15.0, 28.0, 0)),
    Some(Led(16.0, 28.0, 0)),
    Some(Led(17.0, 28.0, 0)),
    Some(Led(18.0, 28.0, 0)),
    Some(Led(19.0, 28.0, 0)),
    Some(Led(20.0, 28.0, 0)),
    Some(Led(21.0, 28.0, 0)),
    Some(Led(22.0, 28.0, 0)),
    Some(Led(23.0, 28.0, 0)),
    Some(Led(24.0, 28.0, 0)),
    Some(Led(25.0, 28.0, 0)),
    Some(Led(26.0, 28.0, 0)),
    Some(Led(27.0, 28.0, 0)),
    Some(Led(28.0, 28.0, 0)),
    Some(Led(29.0, 28.0, 0)),
    Some(Led(30.0, 28.0, 0)),
    Some(Led(31.0, 28.0, 0)),
    Some(Led(32.0, 28.0, 0)),
    Some(Led(11.5, 29.0, 0)),
    Some(Led(12.5, 29.0, 0)),
    Some(Led(13.5, 29.0, 0)),
    Some(Led(14.5, 29.0, 0)),
    Some(Led(15.5, 29.0, 0)),
    Some(Led(16.5, 29.0, 0)),
    Some(Led(17.5, 29.0, 0)),
    Some(Led(18.5, 29.0, 0)),
    Some(Led(19.5, 29.0, 0)),
    Some(Led(20.5, 29.0, 0)),
    Some(Led(21.5, 29.0, 0)),
    Some(Led(22.5, 29.0, 0)),
    Some(Led(23.5, 29.0, 0)),
    Some(Led(24.5, 29.0, 0)),
    Some(Led(25.5, 29.0, 0)),
    Some(Led(26.5, 29.0, 0)),
    Some(Led(27.5, 29.0, 0)),
    Some(Led(28.5, 29.0, 0)),
    Some(Led(29.5, 29.0, 0)),
    Some(Led(30.5, 29.0, 0)),
    Some(Led(31.5, 29.0, 0)),
    None,
    Some(Led(12.0, 30.0, 0)),
    Some(Led(13.0, 30.0, 0)),
    Some(Led(14.0, 30.0, 0)),
    Some(Led(15.0, 30.0, 0)),
    Some(Led(16.0, 30.0, 0)),
    Some(Led(17.0, 30.0, 0)),
    Some(Led(18.0, 30.0, 0)),
    Some(Led(19.0, 30.0, 0)),
    Some(Led(20.0, 30.0, 0)),
    Some(Led(21.0, 30.0, 0)),
    Some(Led(22.0, 30.0, 0)),
    Some(Led(23.0, 30.0, 0)),
    Some(Led(24.0, 30.0, 0)),
    Some(Led(25.0, 30.0, 0)),
    Some(Led(26.0, 30.0, 0)),
    Some(Led(27.0, 30.0, 0)),
    Some(Led(28.0, 30.0, 0)),
    Some(Led(29.0, 30.0, 0)),
    Some(Led(30.0, 30.0, 0)),
    Some(Led(31.0, 30.0, 0)),
    Some(Led(32.0, 30.0, 0)),
    Some(Led(12.5, 31.0, 0)),
    Some(Led(13.5, 31.0, 0)),
    Some(Led(14.5, 31.0, 0)),
    Some(Led(15.5, 31.0, 0)),
    Some(Led(16.5, 31.0, 0)),
    Some(Led(17.5, 31.0, 0)),
    Some(Led(18.5, 31.0, 0)),
    Some(Led(19.5, 31.0, 0)),
    Some(Led(20.5, 31.0, 0)),
    Some(Led(21.5, 31.0, 0)),
    Some(Led(22.5, 31.0, 0)),
    Some(Led(23.5, 31.0, 0)),
    Some(Led(24.5, 31.0, 0)),
    Some(Led(25.5, 31.0, 0)),
    Some(Led(26.5, 31.0, 0)),
    Some(Led(27.5, 31.0, 0)),
    Some(Led(28.5, 31.0, 0)),
    Some(Led(29.5, 31.0, 0)),
    Some(Led(30.5, 31.0, 0)),
    Some(Led(31.5, 31.0, 0)),
    None,
    Some(Led(13.0, 32.0, 0)),
    Some(Led(14.0, 32.0, 0)),
    Some(Led(15.0, 32.0, 0)),
    Some(Led(16.0, 32.0, 0)),
    Some(Led(17.0, 32.0, 0)),
    Some(Led(18.0, 32.0, 0)),
    Some(Led(19.0, 32.0, 0)),
    Some(Led(20.0, 32.0, 0)),
    Some(Led(21.0, 32.0, 0)),
    Some(Led(22.0, 32.0, 0)),
    Some(Led(23.0, 32.0, 0)),
    Some(Led(24.0, 32.0, 0)),
    Some(Led(25.0, 32.0, 0)),
    Some(Led(26.0, 32.0, 0)),
    Some(Led(27.0, 32.0, 0)),
    Some(Led(28.0, 32.0, 0)),
    Some(Led(29.0, 32.0, 0)),
    Some(Led(30.0, 32.0, 0)),
    Some(Led(31.0, 32.0, 0)),
    Some(Led(32.0, 32.0, 0)),
    Some(Led(13.5, 33.0, 0)),
    Some(Led(14.5, 33.0, 0)),
    Some(Led(15.5, 33.0, 0)),
    Some(Led(16.5, 33.0, 0)),
    Some(Led(17.5, 33.0, 0)),
    Some(Led(18.5, 33.0, 0)),
    Some(Led(19.5, 33.0, 0)),
    Some(Led(20.5, 33.0, 0)),
    Some(Led(21.5, 33.0, 0)),
    Some(Led(22.5, 33.0, 0)),
    Some(Led(23.5, 33.0, 0)),
    Some(Led(24.5, 33.0, 0)),
    Some(Led(25.5, 33.0, 0)),
    Some(Led(26.5, 33.0, 0)),
    Some(Led(27.5, 33.0, 0)),
    Some(Led(28.5, 33.0, 0)),
    Some(Led(29.5, 33.0, 0)),
    Some(Led(30.5, 33.0, 0)),
    Some(Led(31.5, 33.0, 0)),
    None,
    Some(Led(14.0, 34.0, 0)),
    Some(Led(15.0, 34.0, 0)),
    Some(Led(16.0, 34.0, 0)),
    Some(Led(17.0, 34.0, 0)),
    Some(Led(18.0, 34.0, 0)),
    Some(Led(19.0, 34.0, 0)),
    Some(Led(20.0, 34.0, 0)),
    Some(Led(21.0, 34.0, 0)),
    Some(Led(22.0, 34.0, 0)),
    Some(Led(23.0, 34.0, 0)),
    Some(Led(24.0, 34.0, 0)),
    Some(Led(25.0, 34.0, 0)),
    Some(Led(26.0, 34.0, 0)),
    Some(Led(27.0, 34.0, 0)),
    Some(Led(28.0, 34.0, 0)),
    Some(Led(29.0, 34.0, 0)),
    Some(Led(30.0, 34.0, 0)),
    Some(Led(31.0, 34.0, 0)),
    Some(Led(32.0, 34.0, 0)),
    Some(Led(14.5, 35.0, 0)),
    Some(Led(15.5, 35.0, 0)),
    Some(Led(16.5, 35.0, 0)),
    Some(Led(17.5, 35.0, 0)),
    Some(Led(18.5, 35.0, 0)),
    Some(Led(19.5, 35.0, 0)),
    Some(Led(20.5, 35.0, 0)),
    Some(Led(21.5, 35.0, 0)),
    Some(Led(22.5, 35.0, 0)),
    Some(Led(23.5, 35.0, 0)),
    Some(Led(24.5, 35.0, 0)),
    Some(Led(25.5, 35.0, 0)),
    Some(Led(26.5, 35.0, 0)),
    Some(Led(27.5, 35.0, 0)),
    Some(Led(28.5, 35.0, 0)),
    Some(Led(29.5, 35.0, 0)),
    Some(Led(30.5, 35.0, 0)),
    Some(Led(31.5, 35.0, 0)),
    None,
    Some(Led(15.0, 36.0, 0)),
    Some(Led(16.0, 36.0, 0)),
    Some(Led(17.0, 36.0, 0)),
    Some(Led(18.0, 36.0, 0)),
    Some(Led(19.0, 36.0, 0)),
    Some(Led(20.0, 36.0, 0)),
    Some(Led(21.0, 36.0, 0)),
    Some(Led(22.0, 36.0, 0)),
    Some(Led(23.0, 36.0, 0)),
    Some(Led(24.0, 36.0, 0)),
    Some(Led(25.0, 36.0, 0)),
    Some(Led(26.0, 36.0, 0)),
    Some(Led(27.0, 36.0, 0)),
    Some(Led(28.0, 36.0, 0)),
    Some(Led(29.0, 36.0, 0)),
    Some(Led(30.0, 36.0, 0)),
    Some(Led(31.0, 36.0, 0)),
    Some(Led(32.0, 36.0, 0)),
    Some(Led(15.5, 37.0, 0)),
    Some(Led(16.5, 37.0, 0)),
    Some(Led(17.5, 37.0, 0)),
    Some(Led(18.5, 37.0, 0)),
    Some(Led(19.5, 37.0, 0)),
    Some(Led(20.5, 37.0, 0)),
    Some(Led(21.5, 37.0, 0)),
    Some(Led(22.5, 37.0, 0)),
    Some(Led(23.5, 37.0, 0)),
    Some(Led(24.5, 37.0, 0)),
    Some(Led(25.5, 37.0, 0)),
    Some(Led(26.5, 37.0, 0)),
    Some(Led(27.5, 37.0, 0)),
    Some(Led(28.5, 37.0, 0)),
    Some(Led(29.5, 37.0, 0)),
    Some(Led(30.5, 37.0, 0)),
    Some(Led(31.5, 37.0, 0)),
    None,
    Some(Led(16.0, 38.0, 0)),
    Some(Led(17.0, 38.0, 0)),
    Some(Led(18.0, 38.0, 0)),
    Some(Led(19.0, 38.0, 0)),
    Some(Led(20.0, 38.0, 0)),
    Some(Led(21.0, 38.0, 0)),
    Some(Led(22.0, 38.0, 0)),
    Some(Led(23.0, 38.0, 0)),
    Some(Led(24.0, 38.0, 0)),
    Some(Led(25.0, 38.0, 0)),
    Some(Led(26.0, 38.0, 0)),
    Some(Led(27.0, 38.0, 0)),
    Some(Led(28.0, 38.0, 0)),
    Some(Led(29.0, 38.0, 0)),
    Some(Led(30.0, 38.0, 0)),
    Some(Led(31.0, 38.0, 0)),
    Some(Led(32.0, 38.0, 0)),
    Some(Led(16.5, 39.0, 0)),
    Some(Led(17.5, 39.0, 0)),
    Some(Led(18.5, 39.0, 0)),
    Some(Led(19.5, 39.0, 0)),
    Some(Led(20.5, 39.0, 0)),
    Some(Led(21.5, 39.0, 0)),
    Some(Led(22.5, 39.0, 0)),
    Some(Led(23.5, 39.0, 0)),
    Some(Led(24.5, 39.0, 0)),
    Some(Led(25.5, 39.0, 0)),
    Some(Led(26.5, 39.0, 0)),
    Some(Led(27.5, 39.0, 0)),
    Some(Led(28.5, 39.0, 0)),
    Some(Led(29.5, 39.0, 0)),
    Some(Led(30.5, 39.0, 0)),
    Some(Led(31.5, 39.0, 0)),
    None,
    Some(Led(17.0, 40.0, 0)),
    Some(Led(18.0, 40.0, 0)),
    Some(Led(19.0, 40.0, 0)),
    Some(Led(20.0, 40.0, 0)),
    Some(Led(21.0, 40.0, 0)),
    Some(Led(22.0, 40.0, 0)),
    Some(Led(23.0, 40.0, 0)),
    Some(Led(24.0, 40.0, 0)),
    Some(Led(25.0, 40.0, 0)),
    Some(Led(26.0, 40.0, 0)),
    Some(Led(27.0, 40.0, 0)),
    Some(Led(28.0, 40.0, 0)),
    Some(Led(29.0, 40.0, 0)),
    Some(Led(30.0, 40.0, 0)),
    Some(Led(31.0, 40.0, 0)),
    Some(Led(32.0, 40.0, 0)),
    Some(Led(17.5, 41.0, 0)),
    Some(Led(18.5, 41.0, 0)),
    Some(Led(19.5, 41.0, 0)),
    Some(Led(20.5, 41.0, 0)),
    Some(Led(21.5, 41.0, 0)),
    Some(Led(22.5, 41.0, 0)),
    Some(Led(23.5, 41.0, 0)),
    Some(Led(24.5, 41.0, 0)),
    Some(Led(25.5, 41.0, 0)),
    Some(Led(26.5, 41.0, 0)),
    Some(Led(27.5, 41.0, 0)),
    Some(Led(28.5, 41.0, 0)),
    Some(Led(29.5, 41.0, 0)),
    Some(Led(30.5, 41.0, 0)),
    Some(Led(31.5, 41.0, 0)),
    None,
    Some(Led(18.0, 42.0, 0)),
    Some(Led(19.0, 42.0, 0)),
    Some(Led(20.0, 42.0, 0)),
    Some(Led(21.0, 42.0, 0)),
    Some(Led(22.0, 42.0, 0)),
    Some(Led(23.0, 42.0, 0)),
    Some(Led(24.0, 42.0, 0)),
    Some(Led(25.0, 42.0, 0)),
    Some(Led(26.0, 42.0, 0)),
    Some(Led(27.0, 42.0, 0)),
    Some(Led(28.0, 42.0, 0)),
    Some(Led(29.0, 42.0, 0)),
    Some(Led(30.0, 42.0, 0)),
    Some(Led(31.0, 42.0, 0)),
    Some(Led(32.0, 42.0, 0)),
    Some(Led(18.5, 43.0, 0)),
    Some(Led(19.5, 43.0, 0)),
    Some(Led(20.5, 43.0, 0)),
    Some(Led(21.5, 43.0, 0)),
    Some(Led(22.5, 43.0, 0)),
    Some(Led(23.5, 43.0, 0)),
    Some(Led(24.5, 43.0, 0)),
    Some(Led(25.5, 43.0, 0)),
    Some(Led(26.5, 43.0, 0)),
    Some(Led(27.5, 43.0, 0)),
    Some(Led(28.5, 43.0, 0)),
    Some(Led(29.5, 43.0, 0)),
    Some(Led(30.5, 43.0, 0)),
    Some(Led(31.5, 43.0, 0)),
    None,
    Some(Led(19.0, 44.0, 0)),
    Some(Led(20.0, 44.0, 0)),
    Some(Led(21.0, 44.0, 0)),
    Some(Led(22.0, 44.0, 0)),
    Some(Led(23.0, 44.0, 0)),
    Some(Led(24.0, 44.0, 0)),
    Some(Led(25.0, 44.0, 0)),
    Some(Led(26.0, 44.0, 0)),
    Some(Led(27.0, 44.0, 0)),
    Some(Led(28.0, 44.0, 0)),
    Some(Led(29.0, 44.0, 0)),
    Some(Led(30.0, 44.0, 0)),
    Some(Led(31.0, 44.0, 0)),
    Some(Led(32.0, 44.0, 0)),
    Some(Led(19.5, 45.0, 0)),
    Some(Led(20.5, 45.0, 0)),
    Some(Led(21.5, 45.0, 0)),
    Some(Led(22.5, 45.0, 0)),
    Some(Led(23.5, 45.0, 0)),
    Some(Led(24.5, 45.0, 0)),
    Some(Led(25.5, 45.0, 0)),
    Some(Led(26.5, 45.0, 0)),
    Some(Led(27.5, 45.0, 0)),
    Some(Led(28.5, 45.0, 0)),
    Some(Led(29.5, 45.0, 0)),
    Some(Led(30.5, 45.0, 0)),
    Some(Led(31.5, 45.0, 0)),
    None,
    Some(Led(20.0, 46.0, 0)),
    Some(Led(21.0, 46.0, 0)),
    Some(Led(22.0, 46.0, 0)),
    Some(Led(23.0, 46.0, 0)),
    Some(Led(24.0, 46.0, 0)),
    Some(Led(25.0, 46.0, 0)),
    Some(Led(26.0, 46.0, 0)),
    Some(Led(27.0, 46.0, 0)),
    Some(Led(28.0, 46.0, 0)),
    Some(Led(29.0, 46.0, 0)),
    Some(Led(30.0, 46.0, 0)),
    Some(Led(31.0, 46.0, 0)),
    Some(Led(32.0, 46.0, 0)),
    Some(Led(20.5, 47.0, 0)),
    Some(Led(21.5, 47.0, 0)),
    Some(Led(22.5, 47.0, 0)),
    Some(Led(23.5, 47.0, 0)),
    Some(Led(24.5, 47.0, 0)),
    Some(Led(25.5, 47.0, 0)),
    Some(Led(26.5, 47.0, 0)),
    Some(Led(27.5, 47.0, 0)),
    Some(Led(28.5, 47.0, 0)),
    Some(Led(29.5, 47.0, 0)),
    Some(Led(30.5, 47.0, 0)),
    Some(Led(31.5, 47.0, 0)),
    None,
    Some(Led(21.0, 48.0, 0)),
    Some(Led(22.0, 48.0, 0)),
    Some(Led(23.0, 48.0, 0)),
    Some(Led(24.0, 48.0, 0)),
    Some(Led(25.0, 48.0, 0)),
    Some(Led(26.0, 48.0, 0)),
    Some(Led(27.0, 48.0, 0)),
    Some(Led(28.0, 48.0, 0)),
    Some(Led(29.0, 48.0, 0)),
    Some(Led(30.0, 48.0, 0)),
    Some(Led(31.0, 48.0, 0)),
    Some(Led(32.0, 48.0, 0)),
    Some(Led(21.5, 49.0, 0)),
    Some(Led(22.5, 49.0, 0)),
    Some(Led(23.5, 49.0, 0)),
    Some(Led(24.5, 49.0, 0)),
    Some(Led(25.5, 49.0, 0)),
    Some(Led(26.5, 49.0, 0)),
    Some(Led(27.5, 49.0, 0)),
    Some(Led(28.5, 49.0, 0)),
    Some(Led(29.5, 49.0, 0)),
    Some(Led(30.5, 49.0, 0)),
    Some(Led(31.5, 49.0, 0)),
    None,
    Some(Led(22.0, 50.0, 0)),
    Some(Led(23.0, 50.0, 0)),
    Some(Led(24.0, 50.0, 0)),
    Some(Led(25.0, 50.0, 0)),
    Some(Led(26.0, 50.0, 0)),
    Some(Led(27.0, 50.0, 0)),
    Some(Led(28.0, 50.0, 0)),
    Some(Led(29.0, 50.0, 0)),
    Some(Led(30.0, 50.0, 0)),
    Some(Led(31.0, 50.0, 0)),
    Some(Led(32.0, 50.0, 0)),
    Some(Led(22.5, 51.0, 0)),
    Some(Led(23.5, 51.0, 0)),
    Some(Led(24.5, 51.0, 0)),
    Some(Led(25.5, 51.0, 0)),
    Some(Led(26.5, 51.0, 0)),
    Some(Led(27.5, 51.0, 0)),
    Some(Led(28.5, 51.0, 0)),
    Some(Led(29.5, 51.0, 0)),
    Some(Led(30.5, 51.0, 0)),
    Some(Led(31.5, 51.0, 0)),
    None,
    Some(Led(23.0, 52.0, 0)),
    Some(Led(24.0, 52.0, 0)),
    Some(Led(25.0, 52.0, 0)),
    Some(Led(26.0, 52.0, 0)),
    Some(Led(27.0, 52.0, 0)),
    Some(Led(28.0, 52.0, 0)),
    Some(Led(29.0, 52.0, 0)),
    Some(Led(30.0, 52.0, 0)),
    Some(Led(31.0, 52.0, 0)),
    Some(Led(32.0, 52.0, 0)),
    Some(Led(23.5, 53.0, 0)),
    Some(Led(24.5, 53.0, 0)),
    Some(Led(25.5, 53.0, 0)),
    Some(Led(26.5, 53.0, 0)),
    Some(Led(27.5, 53.0, 0)),
    Some(Led(28.5, 53.0, 0)),
    Some(Led(29.5, 53.0, 0)),
    Some(Led(30.5, 53.0, 0)),
    Some(Led(31.5, 53.0, 0)),
    None,
    Some(Led(24.0, 54.0, 0)),
    Some(Led(25.0, 54.0, 0)),
    Some(Led(26.0, 54.0, 0)),
    Some(Led(27.0, 54.0, 0)),
    Some(Led(28.0, 54.0, 0)),
    Some(Led(29.0, 54.0, 0)),
    Some(Led(30.0, 54.0, 0)),
    Some(Led(31.0, 54.0, 0)),
    Some(Led(32.0, 54.0, 0)),
];

#[cfg(test)]
mod tests {
    use crate::anime_image::*;

    #[test]
    fn led_positions() {
        let leds = AniMeImage::generate();
        assert_eq!(leds[0], Some(Led(0.0, 0.0, 0)));
        assert_eq!(leds[1], Some(Led(1.0, 0.0, 0)));
        assert_eq!(leds[2], Some(Led(2.0, 0.0, 0)));
        assert_eq!(leds[32], Some(Led(32.0, 0.0, 0)));
        assert_eq!(leds[33], Some(Led(-0.5, 1.0, 0)));
        assert_eq!(leds[65], Some(Led(31.5, 1.0, 0)));
        assert_eq!(leds[66], None);
        assert_eq!(leds[67], None);
        assert_eq!(leds[68], Some(Led(0.0, 2.0, 0)));
        assert_eq!(leds[100], Some(Led(32.0, 2.0, 0)));
        assert_eq!(leds[101], Some(Led(-0.5, 3.0, 0)));
        assert_eq!(leds[133], Some(Led(31.5, 3.0, 0)));
        assert_eq!(leds[134], None);
        assert_eq!(leds[135], None);
        assert_eq!(leds[136], Some(Led(0.0, 4.0, 0)));
        assert_eq!(leds[168], Some(Led(32.0, 4.0, 0)));
        assert_eq!(leds[169], Some(Led(-0.5, 5.0, 0)));
        assert_eq!(leds[201], Some(Led(31.5, 5.0, 0)));
        assert_eq!(leds[202], None);
        assert_eq!(leds[203], Some(Led(0.0, 6.0, 0)));
        assert_eq!(leds[235], Some(Led(32.0, 6.0, 0)));
        assert_eq!(leds[648], Some(Led(32.0, 20.0, 0))); // end
        assert_eq!(leds[649], Some(Led(7.5, 21.0, 0))); // start
        assert_eq!(leds[673], Some(Led(31.5, 21.0, 0))); // end
    }

    #[test]
    fn led_positions_const() {
        let leds = AniMeImage::generate();
        assert_eq!(leds[1], LED_IMAGE_POSITIONS[1]);
        assert_eq!(leds[34], LED_IMAGE_POSITIONS[34]);
        assert_eq!(leds[69], LED_IMAGE_POSITIONS[69]);
        assert_eq!(leds[137], LED_IMAGE_POSITIONS[137]);
        assert_eq!(leds[169], LED_IMAGE_POSITIONS[169]);
        assert_eq!(leds[170], LED_IMAGE_POSITIONS[170]);
        assert_eq!(leds[236], LED_IMAGE_POSITIONS[236]);
        assert_eq!(leds[649], LED_IMAGE_POSITIONS[649]);
        assert_eq!(leds[674], LED_IMAGE_POSITIONS[674]);
    }

    #[test]
    fn row_starts() {
        assert_eq!(AniMeImage::first_x(5), 0);
        assert_eq!(AniMeImage::first_x(6), 0);
        assert_eq!(AniMeImage::first_x(7), 1);
        assert_eq!(AniMeImage::first_x(8), 1);
        assert_eq!(AniMeImage::first_x(9), 2);
        assert_eq!(AniMeImage::first_x(10), 2);
        assert_eq!(AniMeImage::first_x(11), 3);
    }

    #[test]
    fn row_widths() {
        assert_eq!(AniMeImage::width(5), 33);
        assert_eq!(AniMeImage::width(6), 33);
        assert_eq!(AniMeImage::width(7), 32);
        assert_eq!(AniMeImage::width(8), 32);
        assert_eq!(AniMeImage::width(9), 31);
        assert_eq!(AniMeImage::width(10), 31);
        assert_eq!(AniMeImage::width(11), 30);
        assert_eq!(AniMeImage::width(12), 30);
        assert_eq!(AniMeImage::width(13), 29);
        assert_eq!(AniMeImage::width(14), 29);
        assert_eq!(AniMeImage::width(15), 28);
        assert_eq!(AniMeImage::width(16), 28);
        assert_eq!(AniMeImage::width(17), 27);
        assert_eq!(AniMeImage::width(18), 27);
    }

    #[test]
    fn row_pitch() {
        assert_eq!(AniMeImage::pitch(5), 34);
        assert_eq!(AniMeImage::pitch(6), 33);
        assert_eq!(AniMeImage::pitch(7), 33);
        assert_eq!(AniMeImage::pitch(8), 32);
        assert_eq!(AniMeImage::pitch(9), 32);
        assert_eq!(AniMeImage::pitch(10), 31);
        assert_eq!(AniMeImage::pitch(11), 31);
        assert_eq!(AniMeImage::pitch(12), 30);
        assert_eq!(AniMeImage::pitch(13), 30);
        assert_eq!(AniMeImage::pitch(14), 29);
    }
}
