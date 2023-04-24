use log::{error, info, warn};
use serde_derive::{Deserialize, Serialize};

use crate::{AdvancedAuraType, AuraModeNum, AuraZone};

pub const ASUS_LED_MODE_CONF: &str = "/usr/share/asusd/aura_support.ron";
pub const ASUS_LED_MODE_USER_CONF: &str = "/etc/asusd/asusd_user_ledmodes.ron";
pub const ASUS_KEYBOARD_DEVICES: [&str; 5] = ["1866", "18c6", "1869", "1854", "19b6"];

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LedSupportFile(Vec<LaptopLedData>);

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct LaptopLedData {
    /// Found via `cat /sys/class/dmi/id/board_name`, e.g `GU603ZW`.
    /// The match doesn't have to be the complete model number as it is
    /// typically broken down such:
    /// - GU = product
    /// - 603 = model/platform
    /// - Z = variant/year or perhaps dGPU model (such as RTX 3xxx)
    /// - W = possibly dGPU model (such as RTX 3060Ti)
    pub board_name: String,
    pub layout_name: String,
    pub basic_modes: Vec<AuraModeNum>,
    pub basic_zones: Vec<AuraZone>,
    pub advanced_type: AdvancedAuraType,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LaptopLedData456 {
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
        // let prod_family = dmi.product_family().expect("Could not get
        // product_family");

        if let Some(modes) = LedSupportFile::load_from_config() {
            if let Some(data) = modes.matcher(&board_name) {
                return data;
            }
        }
        info!("Using generic LED control for keyboard brightness only");
        LaptopLedData::default()
    }
}

impl LedSupportFile {
    pub fn get(&self) -> &[LaptopLedData] {
        &self.0
    }

    /// The list is stored in ordered format, so the iterator must be reversed
    /// to ensure we match to *whole names* first before doing a glob match
    pub fn matcher(self, board_name: &str) -> Option<LaptopLedData> {
        for config in self.0.iter().rev() {
            if board_name.contains(&config.board_name) {
                info!("LedSupport: Matched to {}", config.board_name);
                return Some(config.clone());
            }
        }
        None
    }

    pub fn load_from_config() -> Option<Self> {
        let mut loaded = false;
        let mut data = LedSupportFile::default();
        // Load user configs first so they are first to be checked
        if let Ok(file) = std::fs::read_to_string(ASUS_LED_MODE_USER_CONF) {
            if file.is_empty() {
                warn!("{} is empty", ASUS_LED_MODE_USER_CONF);
            } else {
                if let Ok(mut tmp) = ron::from_str::<LedSupportFile>(&file) {
                    data.0.append(&mut tmp.0);
                }
                info!(
                    "Loaded user-defined LED support data from {}",
                    ASUS_LED_MODE_USER_CONF
                );
            }
        }
        // Load and append the default LED support data
        if let Ok(file) = std::fs::read_to_string(ASUS_LED_MODE_CONF) {
            if file.is_empty() {
                warn!("{} is empty", ASUS_LED_MODE_CONF);
            } else {
                let mut tmp: LedSupportFile = ron::from_str(&file)
                    .map_err(|e| error!("{e}"))
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", ASUS_LED_MODE_CONF));
                data.0.append(&mut tmp.0);
                loaded = true;
                info!(
                    "Loaded default LED support data from {}",
                    ASUS_LED_MODE_CONF
                );
            }
        }
        data.0.sort_by(|a, b| a.board_name.cmp(&b.board_name));

        if loaded {
            return Some(data);
        }

        warn!("Does {} exist?", ASUS_LED_MODE_USER_CONF);
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::PathBuf;

    use ron::ser::PrettyConfig;

    use super::LaptopLedData;
    use crate::advanced::LedCode;
    use crate::aura_detection::LedSupportFile;
    // use crate::zoned::Zone;
    use crate::{AdvancedAuraType, AuraModeNum, AuraZone};

    #[test]
    fn check_data_parse() {
        let led = LaptopLedData {
            board_name: "Test".to_owned(),
            layout_name: "ga401".to_owned(),
            basic_modes: vec![AuraModeNum::Static],
            basic_zones: vec![AuraZone::Key1, AuraZone::Logo, AuraZone::BarLeft],
            advanced_type: AdvancedAuraType::Zoned(vec![LedCode::LightbarRight]),
        };

        assert!(ron::to_string(&led).is_ok());
        // assert_eq!(json, String::new());
    }

    #[test]
    fn check_data_file_parse() {
        let mut data = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data.push("data/aura_support.ron");

        let buf = std::fs::read_to_string(&data).unwrap();

        let tmp = ron::from_str::<LedSupportFile>(&buf).unwrap();

        // Ensure the data is sorted
        let mut tmp_sort = tmp.clone();
        tmp_sort.0.sort_by(|a, b| a.board_name.cmp(&b.board_name));
        if tmp != tmp_sort {
            let sorted =
                ron::ser::to_string_pretty(&tmp_sort, PrettyConfig::new().depth_limit(2)).unwrap();
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&data)
                .unwrap();
            file.write_all(sorted.as_bytes()).unwrap();
            panic!(
                "aura_support.ron not sorted, should be {sorted}. File rewritten with correct \
                 order, run test again"
            )
        }

        let my_config = PrettyConfig::new().depth_limit(2);
        println!(
            "RON: {}",
            ron::ser::to_string_pretty(&tmp, my_config).unwrap()
        );
    }
}
