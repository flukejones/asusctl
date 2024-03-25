use rog_platform::hid_raw::HidRaw;
use rog_platform::usb_raw::USBRaw;
use rog_slash::{SlashMode};
use rog_slash::usb::{pkt_set_mode, pkt_set_options, pkts_for_init};

use crate::error::SlashCtlError;

enum Node {
    Usb(USBRaw),
    Hid(HidRaw),
}

impl Node {
    pub fn write_bytes(&self, message: &[u8]) -> Result<(), SlashCtlError> {
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

    // pub fn set_builtins_enabled(&self, enabled: bool) -> Result<(), SlashCtlError> {
        // self.write_bytes(&pkt_set_enable_powersave_anim(enabled))?;
        // self.write_bytes(&pkt_set_enable_display(enabled))?;
        // self.write_bytes(&pkt_set_brightness(bright))?;
        // self.write_bytes(&pkt_set_enable_powersave_anim(enabled))
        // Ok(())
    // }
}

pub struct CtrlSlash {
    // node: HidRaw,
    node: Node,
    // slash_type: SlashType,
    // // set to force thread to exit
    // thread_exit: Arc<AtomicBool>,
    // // Set to false when the thread exits
    // thread_running: Arc<AtomicBool>,
}

impl CtrlSlash {
    #[inline]
    pub fn new() -> Result<CtrlSlash, SlashCtlError> {
        let usb = USBRaw::new(rog_slash::usb::PROD_ID).ok();
        let hid = HidRaw::new(rog_slash::usb::PROD_ID_STR).ok();
        let node = if usb.is_some() {
            unsafe { Node::Usb(usb.unwrap_unchecked()) }
        } else if hid.is_some() {
            unsafe { Node::Hid(hid.unwrap_unchecked().0) }
        } else {
            return Err(SlashCtlError::NotSupported);
        };

        // Maybe, detecting the slash-type may become necessary
        // let slash_type = get_slash_type()?;

        let ctrl = CtrlSlash {
            node,
            // slash_type,
            // thread_exit: Arc::new(AtomicBool::new(false)),
            // thread_running: Arc::new(AtomicBool::new(false)),
        };
        ctrl.do_initialization()?;

        Ok(ctrl)
    }

    fn do_initialization(&self) -> Result<(), SlashCtlError> {

        let init_packets = pkts_for_init();
        self.node.write_bytes(&init_packets[0])?;
        self.node.write_bytes(&init_packets[1])?;

        Ok(())
    }

    pub fn set_options(&self, enabled: bool, brightness: u8, interval: u8) -> Result<(), SlashCtlError> {
        let command_packets = pkt_set_options(enabled, brightness, interval);
        self.node.write_bytes(&command_packets)?;
        Ok(())
    }

    pub fn set_slash_mode(&self, slash_mode: SlashMode) -> Result<(), SlashCtlError> {
        let command_packets = pkt_set_mode(slash_mode as u8);
        self.node.write_bytes(&command_packets[0])?;
        self.node.write_bytes(&command_packets[1])?;
        Ok(())
    }
}