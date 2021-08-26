use log::warn;
use serde_derive::{Deserialize, Serialize};
use zbus::dbus_interface;
use zvariant::ObjectPath;
use zvariant_derive::Type;

use crate::{
    ctrl_anime::CtrlAnime, ctrl_aura::controller::CtrlKbdLed, ctrl_charge::CtrlCharge,
    ctrl_profiles::controller::CtrlPlatformProfile, ctrl_rog_bios::CtrlRogBios, GetSupported,
};

use rog_supported::{
    AnimeSupportedFunctions, ChargeSupportedFunctions, LedSupportedFunctions,
    PlatformProfileFunctions, RogBiosSupportedFunctions,
};

#[derive(Serialize, Deserialize, Type)]
pub struct SupportedFunctions {
    pub anime_ctrl: AnimeSupportedFunctions,
    pub charge_ctrl: ChargeSupportedFunctions,
    pub platform_profile: PlatformProfileFunctions,
    pub keyboard_led: LedSupportedFunctions,
    pub rog_bios_ctrl: RogBiosSupportedFunctions,
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl SupportedFunctions {
    fn supported_functions(&self) -> &SupportedFunctions {
        self
    }
}

impl crate::ZbusAdd for SupportedFunctions {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(
                &ObjectPath::from_str_unchecked("/org/asuslinux/Supported"),
                self,
            )
            .map_err(|err| {
                warn!("SupportedFunctions: add_to_server {}", err);
                err
            })
            .ok();
    }
}

impl GetSupported for SupportedFunctions {
    type A = SupportedFunctions;

    fn get_supported() -> Self::A {
        SupportedFunctions {
            anime_ctrl: CtrlAnime::get_supported(),
            keyboard_led: CtrlKbdLed::get_supported(),
            charge_ctrl: CtrlCharge::get_supported(),
            platform_profile: CtrlPlatformProfile::get_supported(),
            rog_bios_ctrl: CtrlRogBios::get_supported(),
        }
    }
}
