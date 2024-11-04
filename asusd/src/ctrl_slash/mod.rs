pub mod config;
pub mod trait_impls;

use config_traits::{StdConfig, StdConfigLoad};
use log::info;
use rog_platform::hid_raw::HidRaw;
use rog_platform::usb_raw::USBRaw;
use rog_slash::error::SlashError;
use rog_slash::usb::{
    get_maybe_slash_type, pkt_save, pkt_set_mode, pkt_set_options, pkts_for_init,
};
use rog_slash::{SlashMode, SlashType};

use crate::ctrl_slash::config::SlashConfig;
use crate::error::RogError;

enum Node {
    Usb(USBRaw),
    Hid(HidRaw),
}

impl Node {
    pub fn write_bytes(&self, message: &[u8]) -> Result<(), RogError> {
        let hex_message: String = message.iter().map(|byte| format!("{:02x}", byte)).collect();
        // TODO: map and pass on errors
        match self {
            Node::Usb(u) => {
                u.write_bytes(message).ok();
            }
            Node::Hid(h) => {
                h.write_bytes(message).unwrap();
            }
        }
        println!("bytes: {}", hex_message);
        Ok(())
    }
}

pub struct CtrlSlash {
    node: Node,
    config: SlashConfig,
    slash_type: SlashType,
}

impl CtrlSlash {
    #[inline]
    pub fn new() -> Result<CtrlSlash, RogError> {
        let slash_type = get_maybe_slash_type()?;
        if matches!(slash_type, SlashType::Unsupported) {
            info!("No Slash capable laptop found");
            return Err(RogError::Slash(SlashError::NoDevice));
        }

        let hid = HidRaw::new(slash_type.prod_id_str()).ok();
        dbg!(&hid);
        if let Some(hid) = &hid {
            for pkt in &rog_slash::usb::pkts_for_init(rog_slash::SlashType::GA605) {
                hid.write_bytes(pkt)?;
            }
            let option_packets =
                rog_slash::usb::pkt_set_options(rog_slash::SlashType::GA605, true, 5, 5);
            hid.write_bytes(&option_packets)?;

            let mode_packets = rog_slash::usb::pkt_set_mode(
                rog_slash::SlashType::GA605,
                rog_slash::SlashMode::BitStream,
            );
            hid.write_bytes(&mode_packets[1])?;
        }
        let hid = HidRaw::new(slash_type.prod_id_str()).ok();
        dbg!(&hid);

        let usb = USBRaw::new(slash_type.prod_id()).ok();
        let node = if hid.is_some() {
            unsafe { Node::Hid(hid.unwrap_unchecked()) }
        } else if usb.is_some() {
            unsafe { Node::Usb(usb.unwrap_unchecked()) }
        } else {
            return Err(RogError::Slash(SlashError::NoDevice));
        };

        let ctrl = CtrlSlash {
            node,
            slash_type,
            config: SlashConfig::new().load(),
        };
        ctrl.do_initialization()?;

        Ok(ctrl)
    }

    fn do_initialization(&self) -> Result<(), RogError> {
        for pkt in &pkts_for_init(self.slash_type) {
            self.node.write_bytes(pkt)?;
        }

        // Apply config upon initialization
        let option_packets = pkt_set_options(
            self.slash_type,
            self.config.slash_enabled,
            self.config.slash_brightness,
            self.config.slash_interval,
        );
        self.node.write_bytes(&option_packets)?;

        let mode_packets = pkt_set_mode(self.slash_type, self.config.slash_mode);
        // self.node.write_bytes(&mode_packets[0])?;
        self.node.write_bytes(&mode_packets[1])?;

        Ok(())
    }

    pub fn set_options(&self, enabled: bool, brightness: u8, interval: u8) -> Result<(), RogError> {
        let command_packets = pkt_set_options(self.slash_type, enabled, brightness, interval);
        self.node.write_bytes(&command_packets)?;
        Ok(())
    }

    pub fn set_slash_mode(&self, slash_mode: SlashMode) -> Result<(), RogError> {
        let command_packets = pkt_set_mode(self.slash_type, slash_mode);
        // self.node.write_bytes(&command_packets[0])?;
        self.node.write_bytes(&command_packets[1])?;
        self.node.write_bytes(&pkt_save(self.slash_type))?;
        Ok(())
    }
}
