use log::{info, warn};
use rog_aura::{AuraModeNum, AuraZone};
use serde_derive::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Read;

pub const ASUS_LED_MODE_CONF: &str = "/etc/asusd/asusd-ledmodes.toml";
pub const ASUS_LED_MODE_USER_CONF: &str = "/etc/asusd/asusd-user-ledmodes.toml";
pub const ASUS_KEYBOARD_DEVICES: [&str; 4] = ["1866", "1869", "1854", "19b6"];

pub fn print_board_info() {
    let dmi = sysfs_class::DmiId::default();
    let board_name = dmi.board_name().expect("Could not get board_name");
    let prod_family = dmi.product_family().expect("Could not get product_family");

    info!("Product family: {}", prod_family.trim());
    info!("Board name: {}", board_name.trim());
}

pub fn print_modes(supported_modes: &[u8]) {
    if !supported_modes.is_empty() {
        info!("Supported Keyboard LED modes are:");
        for mode in supported_modes {
            let mode = <&str>::from(&<AuraModeNum>::from(*mode));
            info!("- {}", mode);
        }
        info!(
            "If these modes are incorrect you can edit {}",
            ASUS_LED_MODE_CONF
        );
    } else {
        info!("No RGB control available");
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct LedSupportFile {
    led_data: Vec<LaptopLedData>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LaptopLedData {
    pub prod_family: String,
    pub board_names: Vec<String>,
    pub standard: Vec<AuraModeNum>,
    pub multizone: Vec<AuraZone>,
    pub per_key: bool,
}

impl LaptopLedData {
    pub fn get_data() -> Self {
        let dmi = sysfs_class::DmiId::default();
        let board_name = dmi.board_name().expect("Could not get board_name");
        let prod_family = dmi.product_family().expect("Could not get product_family");

        if let Some(modes) = LedSupportFile::load_from_config() {
            if let Some(data) = modes.matcher(&prod_family, &board_name) {
                return data;
            }
        }
        info!("Using generic LED control for keyboard brightness only");
        LaptopLedData {
            prod_family,
            board_names: vec![board_name],
            standard: vec![],
            multizone: vec![],
            per_key: false,
        }
    }
}

impl LedSupportFile {
    /// Consumes the LEDModes
    fn matcher(self, prod_family: &str, board_name: &str) -> Option<LaptopLedData> {
        for config in self.led_data {
            if prod_family.contains(&config.prod_family) {
                for board in &config.board_names {
                    if board_name.contains(board) {
                        info!("Matched to {} {}", config.prod_family, board);
                        return Some(config);
                    }
                }
            }
        }
        None
    }

    fn load_from_config() -> Option<Self> {
        let mut loaded = false;
        let mut data = LedSupportFile::default();
        // Load user configs first so they are first to be checked
        if let Ok(mut file) = OpenOptions::new().read(true).open(&ASUS_LED_MODE_USER_CONF) {
            let mut buf = String::new();
            if let Ok(l) = file.read_to_string(&mut buf) {
                if l == 0 {
                    warn!("{} is empty", ASUS_LED_MODE_USER_CONF);
                } else {
                    if let Ok(mut tmp) = toml::from_str::<LedSupportFile>(&buf) {
                        data.led_data.append(&mut tmp.led_data);
                    }
                    info!(
                        "Loaded user-defined LED support data from {}",
                        ASUS_LED_MODE_USER_CONF
                    );
                }
            }
        }
        // Load and append the default LED support data
        if let Ok(mut file) = OpenOptions::new().read(true).open(&ASUS_LED_MODE_CONF) {
            let mut buf = String::new();
            if let Ok(l) = file.read_to_string(&mut buf) {
                if l == 0 {
                    warn!("{} is empty", ASUS_LED_MODE_CONF);
                } else {
                    let mut tmp: LedSupportFile = toml::from_str(&buf)
                        .unwrap_or_else(|_| panic!("Could not deserialise {}", ASUS_LED_MODE_CONF));
                    data.led_data.append(&mut tmp.led_data);
                    loaded = true;
                    info!(
                        "Loaded default LED support data from {}",
                        ASUS_LED_MODE_CONF
                    );
                }
            }
        }

        if loaded {
            return Some(data);
        }

        warn!("Does {} exist?", ASUS_LED_MODE_USER_CONF);
        None
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::OpenOptions, io::Read, path::PathBuf};

    use super::LaptopLedData;
    use rog_aura::{AuraModeNum, AuraZone};

    #[test]
    fn check_data_parse() {
        let led = LaptopLedData {
            prod_family: "Test".to_owned(),
            board_names: vec!["Test".to_owned()],
            standard: vec![AuraModeNum::Static],
            multizone: vec![AuraZone::Key1, AuraZone::Logo, AuraZone::BarLeft],
            per_key: false,
        };

        let toml = toml::to_string_pretty(&led).unwrap();
        println!("{toml}");

        let mut data = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data.push("../data/asusd-ledmodes.toml");

        let mut file = OpenOptions::new().read(true).open(&data).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();

        let x = toml::to_string_pretty(&buf).unwrap();
        println!("{x}");
    }
}
