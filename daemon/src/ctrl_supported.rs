use log::warn;
use serde_derive::{Deserialize, Serialize};
use zbus::dbus_interface;
use zvariant::ObjectPath;

use crate::{GetSupported, ctrl_anime::CtrlAnime, ctrl_charge::CtrlCharge, ctrl_leds::CtrlKbdLed, ctrl_profiles::controller::CtrlFanAndCpu, ctrl_rog_bios::CtrlRogBios};

use rog_types::supported::{
    AnimeSupportedFunctions, ChargeSupportedFunctions, FanCpuSupportedFunctions,
    LedSupportedFunctions, RogBiosSupportedFunctions,
};

#[derive(Serialize, Deserialize)]
pub struct SupportedFunctions {
    pub anime_ctrl: AnimeSupportedFunctions,
    pub charge_ctrl: ChargeSupportedFunctions,
    pub fan_cpu_ctrl: FanCpuSupportedFunctions,
    pub keyboard_led: LedSupportedFunctions,
    pub rog_bios_ctrl: RogBiosSupportedFunctions,
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl SupportedFunctions {
    fn supported_functions(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
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
            keyboard_led: CtrlKbdLed::get_supported(),
            anime_ctrl: CtrlAnime::get_supported(),
            charge_ctrl: CtrlCharge::get_supported(),
            fan_cpu_ctrl: CtrlFanAndCpu::get_supported(),
            rog_bios_ctrl: CtrlRogBios::get_supported(),
        }
    }
}
