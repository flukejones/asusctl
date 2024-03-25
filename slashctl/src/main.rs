use crate::ctrl_slash::CtrlSlash;
use crate::error::SlashCtlError;

mod ctrl_slash;
mod error;

fn main() -> Result<(), SlashCtlError>{
    // let args: Vec<String> = args().skip(1).collect();

    let ctrl_slash = CtrlSlash::new()?;
    ctrl_slash.set_options(false, 10, 0)?;
    // ctrl_slash.set_options(true, 5, 2)?;
    // ctrl_slash.set_slash_mode(SlashModes::Flow)?;
    Ok(())
}