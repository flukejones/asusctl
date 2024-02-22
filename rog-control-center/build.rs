use std::path::PathBuf;
use std::str::FromStr;
// use std::fs::OpenOptions;
// use std::io::Write;
// use diter_protocol::ParameterDefinitions;
// use ron::ser::PrettyConfig;

// const LOCALE_EN: &str =
// include_str!("../data/localization/en/parameters.json"); const LOCALE_IT:
// &str = include_str!("../data/localization/it/parameters.json");
// const LOCALE_ZH: &str =
// include_str!("../data/localization/zh/parameters.json");

// fn write_locales() {
//     let root = env!("CARGO_MANIFEST_DIR");
//     let mut path = PathBuf::from_str(root).unwrap();
//     path.push("src/locales.ron");
//     let mut file = OpenOptions::new();
//     file.truncate(true).create(true).write(true);

//     let en: ParameterDefinitions = serde_json::from_str(LOCALE_EN).unwrap();
//     let mut writer = file.open(path).unwrap();

//     let en = ron::ser::to_string_pretty(&en,
// PrettyConfig::new().depth_limit(4)).unwrap();     writer.write_all(en.
// to_string().as_bytes()).unwrap();

//     // let it: ParameterDefinitions =
// serde_json::from_str(LOCALE_IT).unwrap();     // let zh: ParameterDefinitions
// = serde_json::from_str(LOCALE_ZH).unwrap(); }

fn main() {
    // write_locales();

    let root = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from_str(root).unwrap();
    path.push("ui/main_window.slint");
    slint_build::compile(path).unwrap();
}
