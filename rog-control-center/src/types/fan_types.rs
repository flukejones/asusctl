use rog_platform::platform::PlatformProfile;
use rog_profiles::FanCurvePU;

use crate::{FanType, Profile};

impl From<Profile> for PlatformProfile {
    fn from(value: Profile) -> Self {
        match value {
            Profile::Balanced => PlatformProfile::Balanced,
            Profile::Performance => PlatformProfile::Performance,
            Profile::Quiet => PlatformProfile::Quiet,
            Profile::LowPower => PlatformProfile::LowPower,
        }
    }
}

impl From<PlatformProfile> for Profile {
    fn from(value: PlatformProfile) -> Self {
        match value {
            PlatformProfile::Balanced => Profile::Balanced,
            PlatformProfile::Performance => Profile::Performance,
            PlatformProfile::Quiet => Profile::Quiet,
            PlatformProfile::LowPower => Profile::LowPower,
        }
    }
}

impl From<FanType> for FanCurvePU {
    fn from(value: FanType) -> Self {
        match value {
            FanType::CPU => FanCurvePU::CPU,
            FanType::Middle => FanCurvePU::MID,
            FanType::GPU => FanCurvePU::GPU,
        }
    }
}

impl From<FanCurvePU> for FanType {
    fn from(value: FanCurvePU) -> Self {
        match value {
            FanCurvePU::CPU => FanType::CPU,
            FanCurvePU::GPU => FanType::GPU,
            FanCurvePU::MID => FanType::Middle,
        }
    }
}
