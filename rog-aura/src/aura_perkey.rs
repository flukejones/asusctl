/// A `KeyColourArray` contains all data to change the full set of keyboard
/// key colours individually.
///
/// Each row of the internal array is a full HID packet that can be sent
/// to the keyboard EC. One row controls one group of keys, these keys are not
/// necessarily all on the same row of the keyboard, with some splitting between
/// two rows.
#[derive(Clone)]
pub struct KeyColourArray([[u8; 64]; 11]);
impl Default for KeyColourArray {
    fn default() -> Self {
        Self::new()
    }
}
impl KeyColourArray {
    pub fn new() -> Self {
        let mut set = [[0u8; 64]; 11];
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

    /// Initialise and clear the keyboard for custom effects
    #[inline]
    pub const fn get_init_msg() -> [u8; 64] {
        let mut init = [0u8; 64];
        init[0] = 0x5d; // Report ID
        init[1] = 0xbc; // Mode = custom??, 0xb3 is builtin
        init
    }

    #[inline]
    pub fn set(&mut self, key: Key, r: u8, g: u8, b: u8) {
        if let Some((rr, gg, bb)) = self.key(key) {
            *rr = r;
            *gg = g;
            *bb = b;
        }
    }

    /// Indexes in to `KeyColourArray` at the correct row and column
    /// to set a series of three bytes to the chosen R,G,B values
    pub fn key(&mut self, key: Key) -> Option<(&mut u8, &mut u8, &mut u8)> {
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
            Key::BkSpc1 => (3, 30),
            Key::BkSpc2 => (3, 33),
            Key::BkSpc3 => (3, 36),
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
            Key::Ret1 => (6, 12),
            Key::Ret2 => (6, 15),
            Key::Ret3 => (6, 18),
            Key::PgDn => (6, 21),
            Key::LShift => (6, 36),
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
            Key::Rshift1 => (7, 27),
            Key::Rshift2 => (7, 30),
            Key::Rshift3 => (7, 33),
            Key::End => (7, 36),
            Key::LCtrl => (7, 51),
            Key::LFn => (7, 54),
            //
            Key::Meta => (8, 9),
            Key::LAlt => (8, 12),
            Key::Space1 => (8, 15),
            Key::Space2 => (8, 18),
            Key::Space3 => (8, 21),
            Key::Space4 => (8, 24),
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
            Key::None => return None,
        };
        // LOLOLOLOLOLOLOL! Look it's safe okay
        unsafe {
            Some((
                &mut *(&mut self.0[row][col] as *mut u8),
                &mut *(&mut self.0[row][col + 1] as *mut u8),
                &mut *(&mut self.0[row][col + 2] as *mut u8),
            ))
        }
    }

    #[inline]
    pub fn get(&self) -> &[[u8; 64]; 11] {
        &self.0
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Key {
    VolUp,
    VolDown,
    MicMute,
    Rog,
    Esc,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Del,
    Tilde,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N0,
    Hyphen,
    Equals,
    BkSpc1,
    BkSpc2,
    BkSpc3,
    Home,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LBracket,
    RBracket,
    BackSlash,
    PgUp,
    Caps,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    SemiColon,
    Quote,
    Ret1,
    Ret2,
    Ret3,
    PgDn,
    LShift,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    FwdSlash,
    Rshift1,
    Rshift2,
    Rshift3,
    End,
    LCtrl,
    LFn,
    Meta,
    LAlt,
    Space1,
    Space2,
    Space3,
    Space4,
    RAlt,
    PrtSc,
    RCtrl,
    Up,
    Down,
    Left,
    Right,
    RFn,
    None,
}

pub trait KeyLayout {
    fn get_rows(&self) -> &Vec<[Key; 17]>;
}

#[allow(clippy::upper_case_acronyms)]
pub struct GX502Layout(Vec<[Key; 17]>);

impl KeyLayout for GX502Layout {
    fn get_rows(&self) -> &Vec<[Key; 17]> {
        &self.0
    }
}

impl Default for GX502Layout {
    fn default() -> Self {
        GX502Layout(vec![
            [
                Key::None,
                Key::None,
                Key::VolDown,
                Key::VolUp,
                Key::MicMute,
                Key::Rog,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
            ],
            [
                Key::Esc,
                Key::None,
                Key::F1,
                Key::F2,
                Key::F3,
                Key::F4,
                Key::None, // not sure which key to put here
                Key::F5,
                Key::F6,
                Key::F7,
                Key::F8,
                Key::F9,
                Key::F9,
                Key::F10,
                Key::F11,
                Key::F12,
                Key::Del,
            ],
            [
                Key::Tilde,
                Key::N1,
                Key::N2,
                Key::N3,
                Key::N4,
                Key::N5,
                Key::N6,
                Key::N7,
                Key::N8,
                Key::N9,
                Key::N0,
                Key::Hyphen,
                Key::Equals,
                Key::BkSpc1,
                Key::BkSpc2,
                Key::BkSpc3,
                Key::Home,
            ],
            [
                Key::Tab,
                Key::Q,
                Key::W,
                Key::E,
                Key::R,
                Key::T,
                Key::Y,
                Key::U,
                Key::I,
                Key::O,
                Key::P,
                Key::LBracket,
                Key::RBracket,
                Key::BackSlash,
                Key::BackSlash,
                Key::BackSlash,
                Key::PgUp,
            ],
            [
                Key::Caps,
                Key::A,
                Key::S,
                Key::D,
                Key::F,
                Key::G,
                Key::H,
                Key::J,
                Key::K,
                Key::L,
                Key::SemiColon,
                Key::Quote,
                Key::Quote,
                Key::Ret1,
                Key::Ret2,
                Key::Ret3,
                Key::PgDn,
            ],
            [
                Key::LShift,
                Key::LShift,
                Key::Z,
                Key::X,
                Key::C,
                Key::V,
                Key::B,
                Key::N,
                Key::M,
                Key::Comma,
                Key::Period,
                Key::FwdSlash,
                Key::FwdSlash,
                Key::Rshift1,
                Key::Rshift2,
                Key::Rshift3,
                Key::End,
            ],
            [
                Key::LCtrl,
                Key::LFn,
                Key::Meta,
                Key::LAlt,
                Key::Space1,
                Key::Space2,
                Key::Space3,
                Key::Space4,
                Key::Space4,
                Key::RAlt,
                Key::PrtSc,
                Key::RCtrl,
                Key::RCtrl,
                Key::Left,
                Key::Up,
                Key::Right,
                Key::RFn,
            ],
            [
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::None,
                Key::Left,
                Key::Down,
                Key::Right,
                Key::None,
            ],
        ])
    }
}
