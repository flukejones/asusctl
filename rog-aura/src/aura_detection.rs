use dmi_id::DMIID;
use log::{error, info, warn};
use serde_derive::{Deserialize, Serialize};
use typeshare::typeshare;
use zbus::zvariant::Type;

use crate::usb::AuraDevice;
use crate::{AdvancedAuraType, AuraModeNum, AuraZone};

pub const ASUS_LED_MODE_CONF: &str = "/usr/share/asusd/aura_support.ron";
pub const ASUS_LED_MODE_USER_CONF: &str = "/etc/asusd/asusd_user_ledmodes.ron";
pub const ASUS_KEYBOARD_DEVICES: [AuraDevice; 8] = [
    AuraDevice::Tuf,
    AuraDevice::X1854,
    AuraDevice::X1869,
    AuraDevice::X1866,
    AuraDevice::X18c6,
    AuraDevice::X19b6,
    AuraDevice::X1a30,
    AuraDevice::X1abe,
];

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LedSupportFile(Vec<LaptopLedData>);

/// The powerr zones this laptop supports
#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type), zvariant(signature = "s"))]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Copy, Clone)]
pub enum PowerZones {
    /// The logo on some laptop lids
    #[default]
    Logo,
    /// The full keyboard (not zones)
    Keyboard,
    /// The lightbar, typically on the front of the laptop
    Lightbar,
    /// The leds that may be placed around the edge of the laptop lid
    Lid,
    /// The led strip on the rear of some laptops
    RearGlow,
}

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
    pub power_zones: Vec<PowerZones>,
}

impl LaptopLedData {
    pub fn get_data() -> Self {
        let dmi = DMIID::new().unwrap_or_default();
        // let prod_family = dmi.product_family().expect("Could not get
        // product_family");

        if let Some(modes) = LedSupportFile::load_from_supoprt_db() {
            if let Some(data) = modes.matcher(&dmi.board_name) {
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

    /// Load `LedSupportFile` from the `aura_support.ron` file at
    /// `/usr/share/asusd/aura_support.ron` and append with data from
    /// `/etc/asusd/asusd_user_ledmodes.ron` if that file is available.
    ///
    /// Returns `None` if neither file exists or does not parse correctly.
    pub fn load_from_supoprt_db() -> Option<Self> {
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
    use crate::aura_detection::{LedSupportFile, PowerZones};
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
            power_zones: vec![PowerZones::Keyboard, PowerZones::RearGlow],
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
