use async_trait::async_trait;
use serde_derive::{Deserialize, Serialize};
use zbus::zvariant::Type;
use zbus::{dbus_interface, Connection};

use crate::ctrl_anime::CtrlAnime;
use crate::ctrl_aura::controller::CtrlKbdLed;
use crate::ctrl_platform::CtrlPlatform;
use crate::ctrl_power::CtrlPower;
use crate::ctrl_profiles::controller::CtrlPlatformProfile;
use crate::GetSupported;

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct SupportedFunctions(rog_platform::supported::SupportedFunctions);

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl SupportedFunctions {
    pub fn supported_functions(&self) -> &rog_platform::supported::SupportedFunctions {
        &self.0
    }
}

#[async_trait]
impl crate::ZbusRun for SupportedFunctions {
    async fn add_to_server(self, server: &mut Connection) {
        Self::add_to_server_helper(self, "/org/asuslinux/Supported", server).await;
    }
}

impl GetSupported for SupportedFunctions {
    type A = SupportedFunctions;

    fn get_supported() -> Self::A {
        Self(rog_platform::supported::SupportedFunctions {
            anime_ctrl: CtrlAnime::get_supported(),
            keyboard_led: CtrlKbdLed::get_supported(),
            charge_ctrl: CtrlPower::get_supported(),
            platform_profile: CtrlPlatformProfile::get_supported(),
            rog_bios_ctrl: CtrlPlatform::get_supported(),
        })
    }
}
