use rog_anime::error::AnimeError;
use rog_platform::error::PlatformError;
use rog_profiles::error::ProfileError;
use std::convert::From;
use std::fmt;

#[derive(Debug)]
pub enum RogError {
    ParseVendor,
    ParseLed,
    MissingProfile(String),
    Udev(String, std::io::Error),
    Path(String, std::io::Error),
    Read(String, std::io::Error),
    Write(String, std::io::Error),
    NotSupported,
    NotFound(String),
    DoTask(String),
    MissingFunction(String),
    MissingLedBrightNode(String, std::io::Error),
    ReloadFail(String),
    Profiles(ProfileError),
    Initramfs(String),
    Modprobe(String),
    Io(std::io::Error),
    Zbus(zbus::Error),
    ChargeLimit(u8),
    AuraEffectNotSupported,
    NoAuraKeyboard,
    NoAuraNode,
    Anime(AnimeError),
    Platform(PlatformError),
}

impl fmt::Display for RogError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RogError::ParseVendor => write!(f, "Parse gfx vendor error"),
            RogError::ParseLed => write!(f, "Parse LED error"),
            RogError::MissingProfile(profile) => write!(f, "Profile does not exist {}", profile),
            RogError::Udev(deets, error) => write!(f, "udev {}: {}", deets, error),
            RogError::Path(path, error) => write!(f, "Path {}: {}", path, error),
            RogError::Read(path, error) => write!(f, "Read {}: {}", path, error),
            RogError::Write(path, error) => write!(f, "Write {}: {}", path, error),
            RogError::NotSupported => write!(f, "Not supported"),
            RogError::NotFound(deets) => write!(f, "Not found: {}", deets),
            RogError::DoTask(deets) => write!(f, "Task error: {}", deets),
            RogError::MissingFunction(deets) => write!(f, "Missing functionality: {}", deets),
            RogError::MissingLedBrightNode(path, error) => write!(f, "Led node at {} is missing, please check you have the required patch or dkms module installed: {}", path, error),
            RogError::ReloadFail(deets) => write!(f, "Task error: {}", deets),
            RogError::Profiles(deets) => write!(f, "Profile error: {}", deets),
            RogError::Initramfs(detail) => write!(f, "Initiramfs error: {}", detail),
            RogError::Modprobe(detail) => write!(f, "Modprobe error: {}", detail),
            RogError::Io(detail) => write!(f, "std::io error: {}", detail),
            RogError::Zbus(detail) => write!(f, "Zbus error: {}", detail),
            RogError::ChargeLimit(value) => write!(f, "Invalid charging limit, not in range 20-100%: {}", value),
            RogError::AuraEffectNotSupported => write!(f, "Aura effect not supported"),
            RogError::NoAuraKeyboard => write!(f, "No supported Aura keyboard"),
            RogError::NoAuraNode => write!(f, "No Aura keyboard node found"),
            RogError::Anime(deets) => write!(f, "AniMe Matrix error: {}", deets),
            RogError::Platform(deets) => write!(f, "Asus Platform error: {}", deets),
        }
    }
}

impl std::error::Error for RogError {}

impl From<ProfileError> for RogError {
    fn from(err: ProfileError) -> Self {
        RogError::Profiles(err)
    }
}

impl From<AnimeError> for RogError {
    fn from(err: AnimeError) -> Self {
        RogError::Anime(err)
    }
}

impl From<PlatformError> for RogError {
    fn from(err: PlatformError) -> Self {
        RogError::Platform(err)
    }
}

impl From<zbus::Error> for RogError {
    fn from(err: zbus::Error) -> Self {
        RogError::Zbus(err)
    }
}

impl From<std::io::Error> for RogError {
    fn from(err: std::io::Error) -> Self {
        RogError::Io(err)
    }
}

impl From<RogError> for zbus::fdo::Error {
    #[inline]
    fn from(err: RogError) -> Self {
        zbus::fdo::Error::Failed(format!("{}", err))
    }
}
