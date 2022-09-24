#[macro_export]
macro_rules! has_attr {
    ($(#[$doc_comment:meta])? $attr_name:literal $item:ident) => {
        concat_idents::concat_idents!(fn_name = has_, $attr_name {
            $(#[$doc_comment])*
            pub fn fn_name(&self) -> bool {
                match to_device(&self.$item) {
                    Ok(p) => crate::has_attr(&p, $attr_name),
                    Err(_) => false,
                }
            }
        });
    };
}

#[macro_export]
macro_rules! watch_attr {
    ($(#[$doc_comment:meta])? $attr_name:literal $item:ident) => {
        concat_idents::concat_idents!(fn_name = monitor_, $attr_name {
            $(#[$doc_comment])*
            pub fn fn_name(&self) -> Result<inotify::Inotify> {
                let mut path = self.$item.clone();
                path.push($attr_name);
                if let Some(path) = path.to_str() {
                    let mut inotify = inotify::Inotify::init()?;
                    inotify.add_watch(path, inotify::WatchMask::MODIFY)
                        .map_err(|e| {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                PlatformError::AttrNotFound(format!("{}", $attr_name))
                            } else {
                                PlatformError::IoPath(format!("{}", path), e)
                            }
                        })?;
                    return Ok(inotify);
                }
                Err(PlatformError::AttrNotFound(format!("{}", $attr_name)))
            }
        });
    };
}

#[macro_export]
macro_rules! get_attr_bool {
    ($(#[$doc_comment:meta])? $attr_name:literal $item:ident) => {
        concat_idents::concat_idents!(fn_name = get_, $attr_name {
            $(#[$doc_comment])*
            pub fn fn_name(&self) -> Result<bool> {
                crate::read_attr_bool(&to_device(&self.$item)?, $attr_name)
            }
        });
    };
}

#[macro_export]
macro_rules! set_attr_bool {
    ($(#[$doc_comment:meta])? $attr_name:literal $item:ident) => {
        concat_idents::concat_idents!(fn_name = set_, $attr_name {
            $(#[$doc_comment])*
            pub fn fn_name(&self, value: bool) -> Result<()> {
                crate::write_attr_bool(&mut to_device(&self.$item)?, $attr_name, value)
            }
        });
    };
}

#[macro_export]
macro_rules! attr_bool {
    ($attr_name:literal, $item:ident) => {
        crate::has_attr!($attr_name $item);
        crate::get_attr_bool!( $attr_name $item);
        crate::set_attr_bool!($attr_name $item);
        crate::watch_attr!($attr_name $item);
    };
}

#[macro_export]
macro_rules! get_attr_u8 {
    ($(#[$doc_comment:meta])? $attr_name:literal $item:ident) => {
        concat_idents::concat_idents!(fn_name = get_, $attr_name {
            $(#[$doc_comment])*
            pub fn fn_name(&self) -> Result<u8> {
                crate::read_attr_u8(&to_device(&self.$item)?, $attr_name)
            }
        });
    };
}

#[macro_export]
macro_rules! set_attr_u8 {
    ($(#[$doc_comment:meta])? $attr_name:literal $item:ident) => {
        concat_idents::concat_idents!(fn_name = set_, $attr_name {
            $(#[$doc_comment])*
            pub fn fn_name(&self, value: u8) -> Result<()> {
                crate::write_attr_u8(&mut to_device(&self.$item)?, $attr_name, value)
            }
        });
    };
}

#[macro_export]
macro_rules! attr_u8 {
    ($attr_name:literal, $item:ident) => {
        crate::has_attr!($attr_name $item);
        crate::get_attr_u8!($attr_name $item);
        crate::set_attr_u8!($attr_name $item);
        crate::watch_attr!($attr_name $item);
    };
}

#[macro_export]
macro_rules! get_attr_u8_array {
    ($(#[$doc_comment:meta])? $attr_name:literal $item:ident) => {
        concat_idents::concat_idents!(fn_name = get_, $attr_name {
            $(#[$doc_comment])*
            pub fn fn_name(&self) -> Result<Vec<u8>> {
                crate::read_attr_u8_array(&to_device(&self.$item)?, $attr_name)
            }
        });
    };
}

#[macro_export]
macro_rules! set_attr_u8_array {
    ($(#[$doc_comment:meta])? $attr_name:literal $item:ident) => {
        concat_idents::concat_idents!(fn_name = set_, $attr_name {
            $(#[$doc_comment])*
            pub fn fn_name(&self, values: &[u8]) -> Result<()> {
                crate::write_attr_u8_array(&mut to_device(&self.$item)?, $attr_name, values)
            }
        });
    };
}

#[macro_export]
macro_rules! attr_u8_array {
    ($attr_name:literal, $item:ident) => {
        crate::has_attr!($attr_name $item);
        crate::get_attr_u8_array!($attr_name $item);
        crate::set_attr_u8_array!($attr_name $item);
        crate::watch_attr!($attr_name $item);
    };
}
