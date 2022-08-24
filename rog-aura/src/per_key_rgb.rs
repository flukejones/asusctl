use crate::keys::Key;
use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "dbus")]
use zvariant::Type;

/// Represents the per-key raw USB packets
pub type PerKeyRaw = Vec<Vec<u8>>;

/// A `KeyColourArray` contains all data to change the full set of keyboard
/// key colours individually.
///
/// Each row of the internal array is a full HID packet that can be sent
/// to the keyboard EC. One row controls one group of keys, these keys are not
/// necessarily all on the same row of the keyboard, with some splitting between
/// two rows.
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyColourArray(PerKeyRaw);

impl Default for KeyColourArray {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyColourArray {
    pub fn new() -> Self {
        let mut set = vec![vec![0u8; 64]; 11];
        for (count, row) in set.iter_mut().enumerate() {
            row[0] = 0x5d; // Report ID
            row[1] = 0xbc; // Mode = custom??, 0xb3 is builtin
            row[2] = 0x00;
            row[3] = 0x01; // ??
            row[4] = 0x01; // ??, 4,5,6 are normally RGB for builtin mode colours
            row[5] = 0x01; // ??
            row[6] = (count as u8) << 4; // Key group
            if count == 10 {
                row[7] = 0x08; // 0b00001000
            } else {
                row[7] = 0x10; // 0b00010000 addressing? flips for group a0
            }
            row[8] = 0x00;
        }
        KeyColourArray(set)
    }

    /// Initialise and clear the keyboard for custom effects, this must be done for
    /// every time mode switches from builtin to custom
    #[inline]
    pub const fn get_init_msg() -> [u8; 64] {
        let mut init = [0u8; 64];
        init[0] = 0x5d; // Report ID
        init[1] = 0xbc; // Mode = custom??, 0xb3 is builtin
        init
    }

    #[inline]
    pub fn set(&mut self, key: Key, r: u8, g: u8, b: u8) {
        if let Some(c) = self.rgb_for_key(key) {
            c[0] = r;
            c[1] = g;
            c[2] = b;
        }
    }

    /// Indexes in to `KeyColourArray` at the correct row and column
    /// to set a series of three bytes to the chosen R,G,B values
    pub fn rgb_for_key(&mut self, key: Key) -> Option<&mut [u8]> {
        // Tuples are indexes in to array
        let (row, col) = match key {
            Key::VolDown => (0, 15),
            Key::VolUp => (0, 18),
            Key::MicMute => (0, 21),
            Key::Rog => (0, 24),
            //
            Key::Esc => (1, 24),
            Key::F1 => (1, 30),
            Key::F2 => (1, 33),
            Key::F3 => (1, 36),
            Key::F4 => (1, 39),
            Key::F5 => (1, 45),
            Key::F6 => (1, 48),
            Key::F7 => (1, 51),
            Key::F8 => (1, 54),
            //
            Key::F9 => (2, 12),
            Key::F10 => (2, 15),
            Key::F11 => (2, 18),
            Key::F12 => (2, 21),
            Key::Del => (2, 24),
            Key::Tilde => (2, 39),
            Key::N1 => (2, 42),
            Key::N2 => (2, 45),
            Key::N3 => (2, 48),
            Key::N4 => (2, 51),
            Key::N5 => (2, 54),
            //
            Key::N6 => (3, 9),
            Key::N7 => (3, 12),
            Key::N8 => (3, 15),
            Key::N9 => (3, 18),
            Key::N0 => (3, 21),
            Key::Hyphen => (3, 24),
            Key::Equals => (3, 27),
            Key::BkSpc3_1 => (3, 30),
            Key::BkSpc3_2 => (3, 33),
            Key::BkSpc3_3 => (3, 36),
            Key::Home => (3, 39),
            Key::Tab => (3, 54),
            //
            Key::Q => (4, 9),
            Key::W => (4, 12),
            Key::E => (4, 15),
            Key::R => (4, 18),
            Key::T => (4, 21),
            Key::Y => (4, 24),
            Key::U => (4, 27),
            Key::I => (4, 30),
            Key::O => (4, 33),
            Key::P => (4, 36),
            Key::LBracket => (4, 39),
            Key::RBracket => (4, 42),
            Key::BackSlash => (4, 45),
            Key::PgUp => (4, 54),
            //
            Key::Caps => (5, 21),
            Key::A => (5, 24),
            Key::S => (5, 27),
            Key::D => (5, 30),
            Key::F => (5, 33),
            Key::G => (5, 36),
            Key::H => (5, 39),
            Key::J => (5, 42),
            Key::K => (5, 45),
            Key::L => (5, 48),
            Key::SemiColon => (5, 51),
            Key::Quote => (5, 54),
            //
            Key::Return => (6, 9),
            Key::Return3_1 => (6, 12),
            Key::Return3_2 => (6, 15),
            Key::Return3_3 => (6, 18),
            Key::PgDn => (6, 21),
            Key::LShift => (6, 36),
            // TODO: Find correct locations
            Key::LShift3_1 => (6, 36),
            Key::LShift3_2 => (6, 36),
            Key::LShift3_3 => (6, 36),
            Key::Z => (6, 42),
            Key::X => (6, 45),
            Key::C => (6, 48),
            Key::V => (6, 51),
            Key::B => (6, 54),
            //
            Key::N => (7, 9),
            Key::M => (7, 12),
            Key::Comma => (7, 15),
            Key::Period => (7, 18),
            Key::FwdSlash => (7, 21),
            Key::Rshift => (7, 24),
            Key::Rshift3_1 => (7, 27),
            Key::Rshift3_2 => (7, 30),
            Key::Rshift3_3 => (7, 33),
            Key::End => (7, 36),
            Key::LCtrl => (7, 51),
            Key::LFn => (7, 54),
            //
            Key::Meta => (8, 9),
            Key::LAlt => (8, 12),
            Key::Space5_1 => (8, 15),
            Key::Space5_2 => (8, 18),
            Key::Space5_3 => (8, 21),
            Key::Space5_4 => (8, 24),
            Key::Space5_5 => (8, 27),
            Key::RAlt => (8, 30),
            Key::PrtSc => (8, 33),
            Key::RCtrl => (8, 36),
            Key::Up => (8, 42),
            Key::RFn => (8, 51),
            //
            Key::Left => (9, 54),
            //
            Key::Down => (10, 9),
            Key::Right => (10, 12),
            Key::NormalBlank
            | Key::FuncBlank
            | Key::NormalSpacer
            | Key::FuncSpacer
            | Key::ArrowBlank
            | Key::ArrowSpacer
            | Key::UpRegular
            | Key::DownRegular
            | Key::LeftRegular
            | Key::RightRegular
            | Key::UpSplit
            | Key::DownSplit
            | Key::LeftSplit
            | Key::RightSplit
            | Key::ArrowRegularBlank
            | Key::ArrowRegularSpacer
            | Key::ArrowSplitBlank
            | Key::ArrowSplitSpacer
            | Key::RshiftSmall
            | Key::LCtrlMed
            | Key::MediaPlay
            | Key::MediaStop
            | Key::MediaPrev
            | Key::MediaNext
            | Key::Pause
            | Key::NumLock
            | Key::Star
            | Key::NumPadDel
            | Key::NumPadPlus
            | Key::NumPadEnter
            | Key::NumPadPause
            | Key::NumPadPrtSc
            | Key::NumPadHome
            | Key::RCtrlLarge
            | Key::RowEndSpacer => return None,
            Key::Fan | Key::Space | Key::BkSpc => return None,
        };

        Some(&mut self.0[row][col..=col + 2])
    }

    #[inline]
    pub fn get(&self) -> PerKeyRaw {
        self.0.clone()
    }

    #[inline]
    pub fn get_ref(&self) -> &PerKeyRaw {
        &self.0
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut PerKeyRaw {
        &mut self.0
    }
}

pub trait KeyLayout {
    fn get_rows(&self) -> &Vec<[Key; 17]>;
}

impl From<KeyColourArray> for PerKeyRaw {
    fn from(k: KeyColourArray) -> Self {
        k.0
    }
}
