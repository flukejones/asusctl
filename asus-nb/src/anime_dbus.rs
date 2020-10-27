const DBUS_ANIME_PATH : &str = "/org/asuslinux/Anime";
pub const ANIME_PANE1_PREFIX: [u8; 7] =
    [0x5e, 0xc0, 0x02, 0x01, 0x00, 0x73, 0x02];
pub const ANIME_PANE2_PREFIX: [u8; 7] =
    [0x5e, 0xc0, 0x02, 0x74, 0x02, 0x73, 0x02];

use crate::anime_matrix::{AniMeMatrix, AniMePacketType};
use crate::DBUS_NAME;
use dbus::blocking::{Connection, Proxy};
use std::error::Error;
use std::{thread, time::Duration};

use crate::dbus_anime::{
    OrgAsuslinuxDaemon as OrgAsuslinuxDaemonAniMe,
};

/// Interface for the AniMe dot-matrix display
///
/// The resolution is 34x56 (1904) but only 1,215 LEDs in the top-left are used.
/// The display is available only on select GA401 models.
///
/// Actual image ratio when displayed is stretched width.
///
/// Data structure should be nested array of [[u8; 33]; 56]
pub struct AniMeDbusWriter {
    connection: Box<Connection>,
    block_time: u64,
}

impl AniMeDbusWriter {
    #[inline]
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let connection = Connection::new_system()?;
        Ok(AniMeDbusWriter {
            connection: Box::new(connection),
            block_time: 25,
        })
    }

    // Create D-Bus proxy
    fn new_proxy(&self) -> Proxy<&Connection>{
        self.connection.with_proxy(
            DBUS_NAME,
            DBUS_ANIME_PATH,
            Duration::from_millis(200),
        )
    }

    fn thread_sleep(&self) {
        thread::sleep(Duration::from_millis(self.block_time));
    }

    pub fn write_image_to_buf(_buf: &mut AniMePacketType, _image_data: &[u8]) {
        unimplemented!("Image format is in progress of being worked out")
    }

    /// Write an Animatrix image
    ///
    /// The expected input here is *two* Vectors, 640 bytes in length.
    /// The two vectors are each one half of the full image write.
    ///
    /// After each write a flush is written, it is assumed that this tells the
    /// device to go ahead and display the written bytes
    ///
    /// # Note: The vectors are expected to contain the full sequence of bytes
    /// as follows
    ///
    /// - Write packet 1: 0x5e 0xc0 0x02 0x01 0x00 0x73 0x02 .. <led brightness>
    /// - Write packet 2: 0x5e 0xc0 0x02 0x74 0x02 0x73 0x02 .. <led brightness>
    ///
    /// Where led brightness is 0..255, low to high
    #[inline]
    pub fn write_image(&self, image: &mut AniMePacketType)
                       -> Result<(), Box<dyn Error>> {
        let proxy = self.new_proxy();

        image[0][..7].copy_from_slice(&ANIME_PANE1_PREFIX);
        image[1][..7].copy_from_slice(&ANIME_PANE2_PREFIX);

        proxy.set_anime(vec![image[0].to_vec(), image[1].to_vec()])?;
        self.thread_sleep();

        Ok(())
    }

    #[inline]
    pub fn set_leds_brightness(&self, led_brightness: u8)
                               -> Result<(), Box<dyn Error>> {
        let mut anime_matrix = AniMeMatrix::new();

        anime_matrix.fill_with(led_brightness);
        self.write_image(&mut AniMePacketType::from(anime_matrix))?;

        Ok(())
    }

    #[inline]
    pub fn turn_on_off(&self, status: bool) -> Result<(), Box<dyn Error>> {
        let proxy = self.new_proxy();

        proxy.set_on_off(status)?;
        self.thread_sleep();

        Ok(())
    }

    #[inline]
    pub fn turn_boot_on_off(&self, status: bool)
                            -> Result<(), Box<dyn Error>> {
        let proxy = self.new_proxy();

        proxy.set_boot_on_off(status)?;
        self.thread_sleep();

        Ok(())
    }
}
