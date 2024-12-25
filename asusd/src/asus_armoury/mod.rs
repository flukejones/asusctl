use rog_platform::firmware_attributes::{AttrType, FirmwareAttributes};
use zbus::Connection;

use crate::error::RogError;

pub mod attr_enum_int;
pub mod attr_enum_str;
pub mod attr_int;

pub async fn start_attributes_zbus(server: &Connection) -> Result<(), RogError> {
    for attr in FirmwareAttributes::new().attributes() {
        match attr.attribute_type() {
            AttrType::MinMax => {
                attr_int::AsusArmouryAttribute::new(attr.clone())
                    .start_tasks(server)
                    .await?;
            }
            AttrType::EnumInt => {
                attr_enum_int::AsusArmouryAttribute::new(attr.clone())
                    .start_tasks(server)
                    .await?;
            }
            AttrType::EnumStr => {
                attr_enum_str::AsusArmouryAttribute::new(attr.clone())
                    .start_tasks(server)
                    .await?;
            }

            AttrType::Unbounded => {}
        }
    }
    Ok(())
}
