use log::{error, info, warn};
use std::fs::read_to_string;
use std::{fs::write, io, path::PathBuf};
use sysfs_class::{PciDevice, SysClass};

pub struct Module {
    pub name: String,
}

impl Module {
    fn parse(line: &str) -> io::Result<Module> {
        let mut parts = line.split(' ');

        let name = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "module name not found"))?;

        Ok(Module {
            name: name.to_string(),
        })
    }

    pub fn all() -> io::Result<Vec<Module>> {
        let mut modules = Vec::new();

        let data = read_to_string("/proc/modules")?;
        for line in data.lines() {
            let module = Module::parse(line)?;
            modules.push(module);
        }

        Ok(modules)
    }
}

pub struct PciBus {
    path: PathBuf,
}

impl PciBus {
    pub fn new() -> io::Result<PciBus> {
        let path = PathBuf::from("/sys/bus/pci");
        if path.is_dir() {
            Ok(PciBus { path })
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "pci directory not found",
            ))
        }
    }

    pub fn rescan(&self) -> io::Result<()> {
        write(self.path.join("rescan"), "1")
    }
}

pub struct GraphicsDevice {
    _id: String,
    functions: Vec<PciDevice>,
}

impl GraphicsDevice {
    pub fn new(id: String, functions: Vec<PciDevice>) -> GraphicsDevice {
        GraphicsDevice { _id: id, functions }
    }

    pub fn exists(&self) -> bool {
        self.functions.iter().any(|func| func.path().exists())
    }

    pub fn unbind(&self) -> Result<(), std::io::Error> {
        for func in self.functions.iter() {
            if func.path().exists() {
                match func.driver() {
                    Ok(driver) => {
                        info!("{}: Unbinding {}", driver.id(), func.id());
                        unsafe {
                            driver.unbind(&func).map_err(|err| {
                                error!("gfx unbind: {}", err);
                                err
                            })?;
                        }
                    }
                    Err(err) => match err.kind() {
                        io::ErrorKind::NotFound => (),
                        _ => {
                            error!("gfx driver: {:?}, {}", func.path(), err);
                            return Err(err);
                        }
                    },
                }
            }
        }
        Ok(())
    }

    pub fn remove(&self) -> Result<(), std::io::Error> {
        for func in self.functions.iter() {
            if func.path().exists() {
                match func.driver() {
                    Ok(driver) => {
                        error!("{}: in use by {}", func.id(), driver.id());
                    }
                    Err(why) => match why.kind() {
                        std::io::ErrorKind::NotFound => {
                            info!("{}: Removing", func.id());
                            unsafe {
                                // ignore errors and carry on
                                if let Err(err) = func.remove() {
                                    error!("gfx remove: {}", err);
                                }
                            }
                        }
                        _ => {
                            error!("Remove device failed");
                        }
                    },
                }
            } else {
                warn!("{}: Already removed", func.id());
            }
        }
        info!("Removed all gfx devices");
        Ok(())
    }
}
