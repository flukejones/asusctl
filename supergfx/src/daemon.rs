use std::{
    env,
    error::Error,
    sync::{Arc, Mutex},
};

use log::{error, info, warn, LevelFilter};
use std::io::Write;
use supergfxctl::{
    config::GfxConfig,
    controller::CtrlGraphics,
    error::GfxError,
    gfx_vendors::GfxVendors,
    special::{get_asus_gsync_gfx_mode, has_asus_gsync_gfx_mode},
    DBUS_DEST_NAME, GFX_CONFIG_PATH,
};
use zbus::{fdo, Connection, ObjectServer};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = env_logger::Builder::new();
    logger
        .target(env_logger::Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .init();

    let is_service = match env::var_os("IS_SUPERGFX_SERVICE") {
        Some(val) => val == "1",
        None => false,
    };

    if !is_service {
        println!("supergfxd schould be only run from the right systemd service");
        println!(
            "do not run in your terminal, if you need an logs please use journalctl -b -u supergfxd"
        );
        println!("supergfxd will now exit");
        return Ok(());
    }

    start_daemon()
}

fn start_daemon() -> Result<(), Box<dyn Error>> {
    // Start zbus server
    let connection = Connection::new_system()?;
    fdo::DBusProxy::new(&connection)?.request_name(
        DBUS_DEST_NAME,
        fdo::RequestNameFlags::ReplaceExisting.into(),
    )?;

    let mut object_server = ObjectServer::new(&connection);

    let config = GfxConfig::load(GFX_CONFIG_PATH.into());
    let enable_gfx_switching = config.gfx_managed;
    let config = Arc::new(Mutex::new(config));

    // Graphics switching requires some checks on boot specifically for g-sync capable laptops
    if enable_gfx_switching {
        match CtrlGraphics::new(config.clone()) {
            Ok(mut ctrl) => {
                // Need to check if a laptop has the dedicated gfx switch
                if has_asus_gsync_gfx_mode() {
                    do_asus_laptop_checks(&ctrl, config)?;
                }

                ctrl.reload()
                    .unwrap_or_else(|err| error!("Gfx controller: {}", err));
                ctrl.add_to_server(&mut object_server);
            }
            Err(err) => {
                error!("Gfx control: {}", err);
            }
        }
    }

    // Loop to check errors and iterate zbus server
    loop {
        if let Err(err) = object_server.try_handle_next() {
            error!("{}", err);
        }
    }
}

fn do_asus_laptop_checks(
    ctrl: &CtrlGraphics,
    config: Arc<Mutex<GfxConfig>>,
) -> Result<(), GfxError> {
    if let Ok(ded) = get_asus_gsync_gfx_mode() {
        if let Ok(config) = config.lock() {
            if ded == 1 {
                warn!("Dedicated GFX toggle is on but driver mode is not nvidia \nSetting to nvidia driver mode");
                let devices = ctrl.devices();
                let bus = ctrl.bus();
                CtrlGraphics::do_mode_setup_tasks(GfxVendors::Nvidia, false, &devices, &bus)?;
            } else if ded == 0 {
                info!("Dedicated GFX toggle is off");
                let devices = ctrl.devices();
                let bus = ctrl.bus();
                CtrlGraphics::do_mode_setup_tasks(config.gfx_mode, false, &devices, &bus)?;
            }
        }
    }
    Ok(())
}
