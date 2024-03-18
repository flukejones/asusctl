use rog_platform::platform::ThrottlePolicy;
use rog_profiles::FanCurvePU;

use crate::{FanType, Profile};

impl From<Profile> for ThrottlePolicy {
    fn from(value: Profile) -> Self {
        match value {
            Profile::Balanced => ThrottlePolicy::Balanced,
            Profile::Performance => ThrottlePolicy::Performance,
            Profile::Quiet => ThrottlePolicy::Quiet,
        }
    }
}

impl From<ThrottlePolicy> for Profile {
    fn from(value: ThrottlePolicy) -> Self {
        match value {
            ThrottlePolicy::Balanced => Profile::Balanced,
            ThrottlePolicy::Performance => Profile::Performance,
            ThrottlePolicy::Quiet => Profile::Quiet,
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
