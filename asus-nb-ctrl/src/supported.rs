use std::convert::TryInto;

use log::warn;
use serde_derive::{Deserialize, Serialize};
use zbus::dbus_interface;

use crate::{
    ctrl_anime::{AnimeSupportedFunctions, CtrlAnimeDisplay},
    ctrl_charge::{ChargeSupportedFunctions, CtrlCharge},
    ctrl_fan_cpu::{CtrlFanAndCPU, FanCpuSupportedFunctions},
    ctrl_leds::{CtrlKbdBacklight, LedSupportedFunctions},
    ctrl_rog_bios::{CtrlRogBios, RogBiosSupportedFunctions},
    GetSupported,
};

#[derive(Serialize, Deserialize)]
pub struct SupportedFunctions {
    anime_ctrl: AnimeSupportedFunctions,
    charge_ctrl: ChargeSupportedFunctions,
    fan_cpu_ctrl: FanCpuSupportedFunctions,
    keyboard_led: LedSupportedFunctions,
    rog_bios_ctrl: RogBiosSupportedFunctions,
}

#[dbus_interface(name = "org.asuslinux.Supported")]
impl SupportedFunctions {
    fn supported_functions(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

impl crate::ZbusAdd for SupportedFunctions {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(&"/org/asuslinux/Supported".try_into().unwrap(), self)
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
            keyboard_led: CtrlKbdBacklight::get_supported(),
            anime_ctrl: CtrlAnimeDisplay::get_supported(),
            charge_ctrl: CtrlCharge::get_supported(),
            fan_cpu_ctrl: CtrlFanAndCPU::get_supported(),
            rog_bios_ctrl: CtrlRogBios::get_supported(),
        }
    }
}
