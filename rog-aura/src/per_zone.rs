use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "dbus")]
use zbus::zvariant::Type;

/// Represents the zoned raw USB packets
pub type ZonedRaw = Vec<u8>;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum PerZone {
    None,
    KeyboardLeft,
    KeyboardCenterLeft,
    KeyboardCenterRight,
    KeyboardRight,
    LightbarRight,
    LightbarRightCorner,
    LightbarRightBottom,
    LightbarLeftBottom,
    LightbarLeftCorner,
    LightbarLeft,
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZonedColourArray(ZonedRaw);

impl Default for ZonedColourArray {
    fn default() -> Self {
        Self::new()
    }
}

impl ZonedColourArray {
    pub fn new() -> Self {
        let mut pkt = vec![0u8; 64];
        pkt[0] = 0x5d; // Report ID
        pkt[1] = 0xbc; // Mode = custom??, 0xb3 is builtin
        pkt[2] = 0x01;
        pkt[3] = 0x01; // ??
        pkt[4] = 0x04; // ??, 4,5,6 are normally RGB for builtin mode colours
        ZonedColourArray(pkt)
    }

    pub fn rgb_for_zone(&mut self, zone: PerZone) -> &mut [u8] {
        match zone {
            PerZone::None | PerZone::KeyboardLeft => &mut self.0[9..=11],
            PerZone::KeyboardCenterLeft => &mut self.0[12..=14],
            PerZone::KeyboardCenterRight => &mut self.0[15..=17],
            PerZone::KeyboardRight => &mut self.0[18..=20],
            // Two sections missing here?
            PerZone::LightbarRight => &mut self.0[27..=29],
            PerZone::LightbarRightCorner => &mut self.0[30..=32],
            PerZone::LightbarRightBottom => &mut self.0[33..=35],
            PerZone::LightbarLeftBottom => &mut self.0[36..=38],
            PerZone::LightbarLeftCorner => &mut self.0[39..=41],
            PerZone::LightbarLeft => &mut self.0[42..=44],
        }
    }

    #[inline]
    pub fn get(&self) -> ZonedRaw {
        self.0.clone()
    }

    #[inline]
    pub fn get_ref(&self) -> &ZonedRaw {
        &self.0
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut ZonedRaw {
        &mut self.0
    }
}

impl From<ZonedColourArray> for ZonedRaw {
    fn from(k: ZonedColourArray) -> Self {
        k.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{PerZone, ZonedColourArray, ZonedRaw};

    macro_rules! colour_check {
        ($zone:expr, $pkt_idx_start:expr) => {
            let mut zone = ZonedColourArray::new();
            let c = zone.rgb_for_zone($zone);
            c[0] = 255;
            c[1] = 255;
            c[2] = 255;

            let pkt: ZonedRaw = zone.get();
            assert_eq!(pkt[$pkt_idx_start], 0xff);
            assert_eq!(pkt[$pkt_idx_start + 1], 0xff);
            assert_eq!(pkt[$pkt_idx_start + 2], 0xff);
        };
    }

    #[test]
    fn zone_to_packet_check() {
        let zone = ZonedColourArray::new();
        let pkt: ZonedRaw = zone.into();
        assert_eq!(pkt[0], 0x5d);
        assert_eq!(pkt[1], 0xbc);
        assert_eq!(pkt[2], 0x01);
        assert_eq!(pkt[3], 0x01);
        assert_eq!(pkt[4], 0x04);

        colour_check!(PerZone::KeyboardLeft, 9);
        colour_check!(PerZone::KeyboardCenterLeft, 12);
        colour_check!(PerZone::KeyboardCenterRight, 15);
        colour_check!(PerZone::KeyboardRight, 18);

        colour_check!(PerZone::LightbarRight, 27);
        colour_check!(PerZone::LightbarRightCorner, 30);
        colour_check!(PerZone::LightbarRightBottom, 33);
        colour_check!(PerZone::LightbarLeftBottom, 36);
        colour_check!(PerZone::LightbarLeftCorner, 39);
        colour_check!(PerZone::LightbarLeft, 42);
    }
}
