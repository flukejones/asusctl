use crate::anime_matrix::AniMePacketType;
use crate::{DBUS_IFACE, DBUS_NAME};
use dbus::channel::Sender;
use dbus::{blocking::Connection, Message};
use std::error::Error;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{thread, time::Duration};

pub const ANIME_PANE1_PREFIX: [u8; 7] = [0x5e, 0xc0, 0x02, 0x01, 0x00, 0x73, 0x02];
pub const ANIME_PANE2_PREFIX: [u8; 7] = [0x5e, 0xc0, 0x02, 0x74, 0x02, 0x73, 0x02];

/// Interface for the AniMe dot-matrix display
///
/// The resolution is 34x56 (1904) but only 1,215 LEDs in the top-left are used.
/// The display is available only on select GA401 models.
///
/// Actual image ration when displayed is stretched width.
///
/// Data structure should be nested array of [[u8; 33]; 56]
pub struct AniMeDbusWriter {
    connection: Box<Connection>,
    block_time: u64,
    stop: Arc<AtomicBool>,
}

impl AniMeDbusWriter {
    #[inline]
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let connection = Connection::new_system()?;
        Ok(AniMeDbusWriter {
            connection: Box::new(connection),
            block_time: 25,
            stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn write_image_to_buf(_buf: &mut AniMePacketType, _image_data: &[u8]) {
        unimplemented!("Image format is in progress of being worked out")
    }

    /// Write an Animatrix image
    ///
    /// The expected input here is *two* Vectors, 640 bytes in length. The two vectors
    /// are each one half of the full image write.
    ///
    /// After each write a flush is written, it is assumed that this tells the device to
    /// go ahead and display the written bytes
    ///
    /// # Note:
    /// The vectors are expected to contain the full sequence of bytes as follows
    ///
    /// - Write packet 1: 0x5e 0xc0 0x02 0x01 0x00 0x73 0x02 .. <led brightness>
    /// - Write packet 2: 0x5e 0xc0 0x02 0x74 0x02 0x73 0x02 .. <led brightness>
    ///
    /// Where led brightness is 0..255, low to high
    #[inline]
    pub fn write_image(&mut self, image: &mut AniMePacketType) -> Result<(), Box<dyn Error>> {
        if image[0][0] != ANIME_PANE1_PREFIX[0] && image[0][6] != ANIME_PANE1_PREFIX[6] {
            image[0][..7].copy_from_slice(&ANIME_PANE1_PREFIX);
        }
        if image[1][0] != ANIME_PANE2_PREFIX[0] && image[1][6] != ANIME_PANE2_PREFIX[6] {
            image[1][..7].copy_from_slice(&ANIME_PANE2_PREFIX);
        }

        let mut msg =
            Message::new_method_call(DBUS_NAME, "/org/asuslinux/Anime", DBUS_IFACE, "SetAnime")?
                .append2(image[0].to_vec(), image[1].to_vec());
        msg.set_no_reply(true);
        self.connection.send(msg).unwrap();
        thread::sleep(Duration::from_millis(self.block_time));
        if self.stop.load(Ordering::Relaxed) {
            panic!("Got signal to stop!");
        }
        Ok(())
    }
}
