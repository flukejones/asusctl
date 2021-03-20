use crate::error::GraphicsError;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use zvariant_derive::Type;

#[derive(Debug, Type, PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum GfxVendors {
    Nvidia,
    Integrated,
    Compute,
    Vfio,
    Hybrid,
}

impl FromStr for GfxVendors {
    type Err = GraphicsError;

    fn from_str(s: &str) -> Result<Self, GraphicsError> {
        match s.to_lowercase().as_str() {
            "nvidia" => Ok(GfxVendors::Nvidia),
            "hybrid" => Ok(GfxVendors::Hybrid),
            "compute" => Ok(GfxVendors::Compute),
            "vfio" => Ok(GfxVendors::Vfio),
            "integrated" => Ok(GfxVendors::Integrated),
            "nvidia\n" => Ok(GfxVendors::Nvidia),
            "hybrid\n" => Ok(GfxVendors::Hybrid),
            "compute\n" => Ok(GfxVendors::Compute),
            "vfio\n" => Ok(GfxVendors::Vfio),
            "integrated\n" => Ok(GfxVendors::Integrated),
            _ => Err(GraphicsError::ParseVendor),
        }
    }
}

impl From<&GfxVendors> for &str {
    fn from(gfx: &GfxVendors) -> &'static str {
        match gfx {
            GfxVendors::Nvidia => "nvidia",
            GfxVendors::Hybrid => "hybrid",
            GfxVendors::Compute => "compute",
            GfxVendors::Vfio => "vfio",
            GfxVendors::Integrated => "integrated",
        }
    }
}

impl From<GfxVendors> for &str {
    fn from(gfx: GfxVendors) -> &'static str {
        (&gfx).into()
    }
}

#[derive(Debug, Type, PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum GfxRequiredUserAction {
    Logout,
    Reboot,
    None,
}

impl From<&GfxRequiredUserAction> for &str {
    fn from(gfx: &GfxRequiredUserAction) -> &'static str {
        match gfx {
            GfxRequiredUserAction::Logout => "logout",
            GfxRequiredUserAction::Reboot => "reboot",
            GfxRequiredUserAction::None => "no action",
        }
    }
}
