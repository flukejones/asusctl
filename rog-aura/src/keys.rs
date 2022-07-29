use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Key {
    VolUp,
    VolDown,
    MicMute,
    Rog,
    Fan,
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
    BkSpc,
    BkSpc3_1,
    BkSpc3_2,
    BkSpc3_3,
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
    Return,
    Return3_1,
    Return3_2,
    Return3_3,
    PgDn,
    LShift,
    LShift3_1,
    LShift3_2,
    LShift3_3,
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
    Star,
    NumPadDel,
    NumPadPlus,
    NumPadEnter,
    NumPadPause,
    NumPadPrtSc,
    NumPadHome,
    NumLock,
    Rshift,
    RshiftSmall,
    Rshift3_1,
    Rshift3_2,
    Rshift3_3,
    End,
    LCtrl,
    LCtrlMed,
    LFn,
    Meta,
    LAlt,
    Space,
    Space5_1,
    Space5_2,
    Space5_3,
    Space5_4,
    Space5_5,
    Pause,
    RAlt,
    PrtSc,
    RCtrl,
    RCtrlLarge,
    Up,
    Down,
    Left,
    Right,
    UpRegular,
    DownRegular,
    LeftRegular,
    RightRegular,
    UpSplit,
    DownSplit,
    LeftSplit,
    RightSplit,
    RFn,
    MediaPlay,
    MediaStop,
    MediaNext,
    MediaPrev,
    NormalBlank,
    /// To be ignored by per-key effects
    NormalSpacer,
    FuncBlank,
    /// To be ignored by per-key effects
    FuncSpacer,
    ArrowBlank,
    /// To be ignored by per-key effects
    ArrowSpacer,
    ArrowRegularBlank,
    /// To be ignored by per-key effects
    ArrowRegularSpacer,
    ArrowSplitBlank,
    /// To be ignored by per-key effects
    ArrowSplitSpacer,
    /// A gap between regular rows and the rightside buttons
    RowEndSpacer,
}

/// Types of shapes of LED on keyboards. The shape is used for visual representations
///
/// A post fix of Spacer *must be ignored by per-key effects
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize)]
pub enum KeyShape {
    Tilde,
    #[default]
    Normal,
    NormalBlank,
    NormalSpacer,
    Func,
    FuncBlank,
    FuncSpacer,
    Space,
    Space5,
    LCtrlMed,
    LShift,
    /// Used in a group of 3 (LED's)
    LShift3,
    RShift,
    RshiftSmall,
    /// Used in a group of 3 (LED's)
    RShift3,
    Return,
    Return3,
    Tab,
    Caps,
    Backspace,
    /// Used in a group of 3 (LED's)
    Backspace3,
    Arrow,
    ArrowBlank,
    ArrowSpacer,
    ArrowSplit,
    ArrowSplitBlank,
    ArrowSplitSpacer,
    ArrowRegularBlank,
    ArrowRegularSpacer,
    RowEndSpacer,
}

impl KeyShape {
    pub const fn ux(&self) -> f32 {
        match self {
            Self::Tilde => 0.8,
            Self::Normal => 1.0,
            Self::NormalBlank => 1.0,
            Self::NormalSpacer => 1.0,
            Self::Func => 1.0,
            Self::FuncBlank => 1.0,
            Self::FuncSpacer => 0.6,
            Self::Space => 5.0,
            Self::Space5 => 1.0,
            Self::LCtrlMed => 1.1,
            Self::LShift => 2.0,
            Self::LShift3 => 0.67,
            Self::RShift => 2.8,
            Self::RshiftSmall => 1.7,
            Self::RShift3 => 0.93,
            Self::Return => 2.2,
            Self::Return3 => 0.7333,
            Self::Tab => 1.4,
            Self::Caps => 1.6,
            Self::Backspace => 2.0,
            Self::Backspace3 => 0.666,
            Self::ArrowRegularBlank | Self::ArrowRegularSpacer => 0.7,
            Self::Arrow => 0.8,
            Self::ArrowBlank | Self::ArrowSpacer => 1.0,
            Self::ArrowSplit | Self::ArrowSplitBlank | Self::ArrowSplitSpacer => 1.0,
            Self::RowEndSpacer => 0.1,
        }
    }
    pub const fn uy(&self) -> f32 {
        match self {
            Self::Func => 0.8,
            Self::RowEndSpacer => 0.1,
            Self::FuncBlank => 0.8,
            Self::FuncSpacer => 0.8,
            Self::Arrow | Self::ArrowBlank | Self::ArrowSpacer => 0.6,
            Self::ArrowSplit | Self::ArrowSplitBlank | Self::ArrowSplitSpacer => 5.0,
            _ => 1.0,
        }
    }

    /// A blank is used to space keys out in GUI's and can be used or ignored
    /// depednign on the per-key effect
    pub const fn is_blank(&self) -> bool {
        match self {
            Self::NormalBlank
            | Self::FuncBlank
            | Self::ArrowBlank
            | Self::ArrowSplitBlank
            | Self::ArrowRegularBlank => true,
            _ => false,
        }
    }

    /// A spacer is used to space keys out in GUI's, but ignored in per-key effects
    pub const fn is_spacer(&self) -> bool {
        match self {
            Self::FuncSpacer
            | Self::NormalSpacer
            | Self::ArrowSpacer
            | Self::ArrowSplitSpacer
            | Self::ArrowRegularSpacer => true,
            _ => false,
        }
    }

    /// All keys with a postfix of some number
    pub const fn is_group(&self) -> bool {
        match self {
            Self::LShift3 | Self::RShift3 => true,
            Self::Return3 | Self::Space5 | Self::Backspace3 => true,
            _ => false,
        }
    }

    pub const fn is_arrow_cluster(&self) -> bool {
        match self {
            Self::Arrow | Self::ArrowBlank | Self::ArrowSpacer => true,
            _ => false,
        }
    }

    pub const fn is_arrow_splits(&self) -> bool {
        match self {
            Self::ArrowSplit | Self::ArrowSplitBlank | Self::ArrowSplitSpacer => true,
            _ => false,
        }
    }
}

impl From<Key> for KeyShape {
    fn from(k: Key) -> Self {
        match k {
            Key::VolUp
            | Key::VolDown
            | Key::MicMute
            | Key::Rog
            | Key::Fan
            | Key::Esc
            | Key::F1
            | Key::F2
            | Key::F3
            | Key::F4
            | Key::F5
            | Key::F6
            | Key::F7
            | Key::F8
            | Key::F9
            | Key::F10
            | Key::F11
            | Key::F12
            | Key::Del => KeyShape::Func,
            Key::Tilde => KeyShape::Tilde,

            Key::BkSpc => KeyShape::Backspace,
            Key::BkSpc3_1 | Key::BkSpc3_2 | Key::BkSpc3_3 => KeyShape::Backspace3,
            Key::Tab | Key::BackSlash => KeyShape::Tab,
            Key::Caps => KeyShape::Caps,

            Key::Return => KeyShape::Return,
            Key::Return3_1 | Key::Return3_2 | Key::Return3_3 => KeyShape::Return3,
            Key::LCtrlMed => KeyShape::LCtrlMed,
            Key::LShift => KeyShape::LShift,

            Key::Rshift | Key::RCtrlLarge => KeyShape::RShift,
            Key::RshiftSmall => KeyShape::RshiftSmall,
            Key::Rshift3_1 | Key::Rshift3_2 | Key::Rshift3_3 => KeyShape::RShift3,

            Key::Space => KeyShape::Space,
            Key::Space5_1 | Key::Space5_2 | Key::Space5_3 | Key::Space5_4 | Key::Space5_5 => {
                KeyShape::Space5
            }

            Key::NumPadPause | Key::NumPadPrtSc | Key::NumPadHome | Key::NumPadDel => {
                KeyShape::Func
            }

            Key::NormalBlank => KeyShape::NormalBlank,
            Key::NormalSpacer => KeyShape::NormalSpacer,

            Key::FuncBlank => KeyShape::FuncBlank,
            Key::FuncSpacer => KeyShape::FuncSpacer,

            Key::Up | Key::Down | Key::Left | Key::Right => KeyShape::Arrow,
            Key::ArrowBlank => KeyShape::ArrowBlank,
            Key::ArrowSpacer => KeyShape::ArrowSpacer,

            Key::ArrowRegularBlank => KeyShape::ArrowRegularBlank,
            Key::ArrowRegularSpacer => KeyShape::ArrowRegularSpacer,

            Key::UpSplit | Key::LeftSplit | Key::DownSplit | Key::RightSplit => {
                KeyShape::ArrowSplit
            }
            Key::ArrowSplitBlank => KeyShape::ArrowSplitBlank,
            Key::ArrowSplitSpacer => KeyShape::ArrowSplitSpacer,

            Key::RowEndSpacer => KeyShape::RowEndSpacer,

            _ => KeyShape::Normal,
        }
    }
}

impl From<&Key> for KeyShape {
    fn from(k: &Key) -> Self {
        (*k).into()
    }
}
