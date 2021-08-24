use std::{fs::OpenOptions, io::Read, path::Path};

use crate::error::GfxError;

static ASUS_SWITCH_GRAPHIC_MODE: &str =
    "/sys/firmware/efi/efivars/AsusSwitchGraphicMode-607005d5-3f75-4b2e-98f0-85ba66797a3e";

pub(crate) fn has_asus_gsync_gfx_mode() -> bool {
    Path::new(ASUS_SWITCH_GRAPHIC_MODE).exists()
}

pub(crate) fn get_asus_gsync_gfx_mode() -> Result<i8, GfxError> {
    let path = ASUS_SWITCH_GRAPHIC_MODE;
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|err| GfxError::Path(path.into(), err))?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|err| GfxError::Read(path.into(), err))?;

    let idx = data.len() - 1;
    Ok(data[idx] as i8)
}