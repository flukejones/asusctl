use asus_nb::aura_modes::{
    AuraModes, BREATHING, COMET, FLASH, HIGHLIGHT, LASER, MULTISTATIC, PULSE, RAIN, RAINBOW, RGB,
    RIPPLE, STAR, STATIC, STROBE,
};
use log::{info, warn};

static HELP_ADDRESS: &str = "https://gitlab.com/asus-linux/asus-nb-ctrl";

pub fn match_laptop() -> LaptopBase {
    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();
        if device_desc.vendor_id() == 0x0b05 {
            match device_desc.product_id() {
                0x1866 => return select_1866_device("1866".to_owned()),
                0x1869 => return select_1866_device("1869".to_owned()),
                0x1854 => {
                    info!("Found GL753 or similar");
                    return LaptopBase {
                        usb_product: "1854".to_string(),
                        supported_modes: vec![STATIC, BREATHING, STROBE],
                    };
                }
                _ => {}
            }
        }
    }
    panic!("could not match laptop");
}

fn select_1866_device(prod: String) -> LaptopBase {
    let dmi = sysfs_class::DmiId::default();
    let board_name = dmi.board_name().expect("Could not get board_name");
    let prod_name = dmi.product_name().expect("Could not get board_name");

    info!("Product name: {}", prod_name.trim());
    info!("Board name: {}", board_name.trim());

    let mut laptop = LaptopBase {
        usb_product: prod,
        supported_modes: vec![],
    };

    // AniMe, no RGB
    if board_name.starts_with("GA401")
        || board_name.starts_with("GA502")
        || board_name.starts_with("GU502")
    {
        info!("No RGB control available");
    // RGB, per-key settings, no zones
    } else if board_name.starts_with("GX502")
        || board_name.starts_with("GX701")
        || board_name.starts_with("G531")
        || board_name.starts_with("GL531")
        || board_name.starts_with("G532")
    {
        laptop.supported_modes = vec![
            STATIC, BREATHING, STROBE, RAINBOW, STAR, RAIN, HIGHLIGHT, LASER, RIPPLE, PULSE, COMET,
            FLASH, RGB,
        ];
    } else if board_name.starts_with("G531") || board_name.starts_with("G731") {
        laptop.supported_modes = vec![
            STATIC,
            BREATHING,
            STROBE,
            RAINBOW,
            STAR,
            RAIN,
            HIGHLIGHT,
            LASER,
            RIPPLE,
            PULSE,
            COMET,
            FLASH,
            MULTISTATIC,
            RGB,
        ];
    // RGB, limited effects, no zones
    } else if board_name.starts_with("G512LI") || board_name.starts_with("G712LI") {
        laptop.supported_modes = vec![STATIC, BREATHING, STROBE, RAINBOW, PULSE];
    // RGB, limited effects, 4-zone RGB
    } else if board_name.starts_with("GM501")
        || board_name.starts_with("GX531")
        || board_name.starts_with("G512")
        || board_name.starts_with("G712")
    {
        laptop.supported_modes = vec![STATIC, BREATHING, STROBE, RAINBOW, PULSE, MULTISTATIC];
    } else {
        warn!(
            "Unsupported laptop, please request support at {}",
            HELP_ADDRESS
        );
        warn!("Continuing with minimal support")
    }

    if !laptop.supported_modes.is_empty() {
        info!("Supported Keyboard LED modes are:");
        for mode in &laptop.supported_modes {
            let mode = <&str>::from(&<AuraModes>::from(*mode));
            info!("- {}", mode);
        }
        info!(
            "If these modes are incorrect or missing please request support at {}",
            HELP_ADDRESS
        );
    }

    laptop
}

pub struct LaptopBase {
    usb_product: String,
    supported_modes: Vec<u8>,
}

impl LaptopBase {
    pub fn usb_product(&self) -> &str {
        &self.usb_product
    }
    pub fn supported_modes(&self) -> &[u8] {
        &self.supported_modes
    }
}
