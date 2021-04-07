use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "zbus")]
use zvariant_derive::Type;

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

/// The minimal serializable data that can be transferred over wire types.
/// Other data structures in `rog_anime` will convert to this.
#[cfg_attr(feature = "zbus", derive(Type))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AniMeDataBuffer(Vec<u8>);

impl Default for AniMeDataBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl AniMeDataBuffer {
    pub fn new() -> Self {
        AniMeDataBuffer(vec![0u8; ANIME_DATA_LEN])
    }

    pub fn get(&self) -> &[u8] {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    pub fn from_vec(input: Vec<u8>) -> Self {
        Self(input)
    }
}

/// The two packets to be written to USB
pub type AniMePacketType = [[u8; 640]; 2];

impl From<AniMeDataBuffer> for AniMePacketType {
    #[inline]
    fn from(anime: AniMeDataBuffer) -> Self {
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
