pub mod config;
pub mod trait_impls;

use config_traits::{StdConfig, StdConfigLoad};
use log::info;
use rog_platform::hid_raw::HidRaw;
use rog_platform::usb_raw::USBRaw;
use rog_slash::error::SlashError;
use rog_slash::usb::{get_maybe_slash_type, pkt_set_mode, pkt_set_options, pkts_for_init};
use rog_slash::{SlashMode, SlashType};

use crate::ctrl_slash::config::SlashConfig;
use crate::error::RogError;

enum Node {
    Usb(USBRaw),
    Hid(HidRaw),
}

impl Node {
    pub fn write_bytes(&self, message: &[u8]) -> Result<(), RogError> {
        // TODO: map and pass on errors
        match self {
            Node::Usb(u) => {
                u.write_bytes(message).ok();
            }
            Node::Hid(h) => {
                h.write_bytes(message).ok();
            }
        }
        Ok(())
    }
}

pub struct CtrlSlash {
    node: Node,
    config: SlashConfig,
}

impl CtrlSlash {
    #[inline]
    pub fn new() -> Result<CtrlSlash, RogError> {
        let slash_type = get_maybe_slash_type()?;
        if matches!(slash_type, SlashType::Unsupported) {
            info!("No Slash capable laptop found");
            return Err(RogError::Slash(SlashError::NoDevice));
        }

        let usb = USBRaw::new(rog_slash::usb::PROD_ID).ok();
        let hid = HidRaw::new(rog_slash::usb::PROD_ID_STR).ok();
        let node = if usb.is_some() {
            info!("Slash is using raw USB");
            unsafe { Node::Usb(usb.unwrap_unchecked()) }
        } else if hid.is_some() {
            info!("Slash is using HIDRAW");
            unsafe { Node::Hid(hid.unwrap_unchecked()) }
        } else {
            return Err(RogError::Slash(SlashError::NoDevice));
        };

        let ctrl = CtrlSlash {
            node,
            config: SlashConfig::new().load(),
        };
        ctrl.do_initialization()?;

        Ok(ctrl)
    }

    fn do_initialization(&self) -> Result<(), RogError> {
        let init_packets = pkts_for_init();
        self.node.write_bytes(&init_packets[0])?;
        self.node.write_bytes(&init_packets[1])?;

        // Apply config upon initialization
        let option_packets = pkt_set_options(
            self.config.slash_enabled,
            self.config.slash_brightness,
            self.config.slash_interval,
        );
        self.node.write_bytes(&option_packets)?;

        let mode_packets = pkt_set_mode(self.config.slash_mode);
        self.node.write_bytes(&mode_packets[0])?;
        self.node.write_bytes(&mode_packets[1])?;

        Ok(())
    }

    pub fn set_options(&self, enabled: bool, brightness: u8, interval: u8) -> Result<(), RogError> {
        let command_packets = pkt_set_options(enabled, brightness, interval);
        self.node.write_bytes(&command_packets)?;
        Ok(())
    }

    pub fn set_slash_mode(&self, slash_mode: SlashMode) -> Result<(), RogError> {
        let command_packets = pkt_set_mode(slash_mode);
        self.node.write_bytes(&command_packets[0])?;
        self.node.write_bytes(&command_packets[1])?;
        Ok(())
    }
}
