use std::path::PathBuf;
use std::str::FromStr;

use slint_build::CompilerConfiguration;

fn main() {
    // write_locales();

    let root = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from_str(root).unwrap();
    path.push("ui/main_window.slint");
    slint_build::compile_with_config(
        path,
        CompilerConfiguration::new().with_style("cosmic-dark".into()),
    )
    .unwrap();
}
