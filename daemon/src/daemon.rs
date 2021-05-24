use daemon::ctrl_leds::controller::{
    CtrlKbdLed, CtrlKbdLedReloader, CtrlKbdLedTask, CtrlKbdLedZbus,
};
use daemon::{
    config::Config, ctrl_supported::SupportedFunctions, laptops::print_board_info, GetSupported,
};
use daemon::{config_anime::AnimeConfig, config_aura::AuraConfig, ctrl_charge::CtrlCharge};
use daemon::{ctrl_anime::*, ctrl_gfx::controller::CtrlGraphics};
use daemon::{
    ctrl_profiles::{controller::CtrlFanAndCpu, zbus::FanAndCpuZbus},
    laptops::LaptopLedData,
};

use daemon::{CtrlTask, Reloadable, ZbusAdd};
use log::LevelFilter;
use log::{error, info, warn};
use rog_dbus::DBUS_NAME;
use rog_types::gfx_vendors::GfxVendors;
use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use daemon::ctrl_rog_bios::CtrlRogBios;
use std::convert::Into;
use zbus::fdo;
use zbus::Connection;
use zvariant::ObjectPath;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = env_logger::Builder::new();
    logger
        .target(env_logger::Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .init();

    info!("      daemon v{}", daemon::VERSION);
    info!("   rog-anime v{}", rog_anime::VERSION);
    info!("    rog-aura v{}", rog_aura::VERSION);
    info!("    rog-dbus v{}", rog_dbus::VERSION);
    info!("rog-profiles v{}", rog_profiles::VERSION);
    info!("   rog-types v{}", rog_types::VERSION);

    start_daemon()?;
    Ok(())
}

/// The actual main loop for the daemon
fn start_daemon() -> Result<(), Box<dyn Error>> {
    let supported = SupportedFunctions::get_supported();
    print_board_info();
    println!("{}", serde_json::to_string_pretty(&supported).unwrap());

    // Collect tasks for task thread
    let mut tasks: Vec<Box<dyn CtrlTask + Send>> = Vec::new();
    // Start zbus server
    let connection = Connection::new_system()?;
    fdo::DBusProxy::new(&connection)?
        .request_name(DBUS_NAME, fdo::RequestNameFlags::ReplaceExisting.into())?;
    let mut object_server = zbus::ObjectServer::new(&connection);

    let config = Config::load();
    let enable_gfx_switching = config.gfx_managed;
    let config = Arc::new(Mutex::new(config));

    supported.add_to_server(&mut object_server);

    match CtrlRogBios::new(config.clone()) {
        Ok(mut ctrl) => {
            // Do a reload of any settings
            ctrl.reload()
                .unwrap_or_else(|err| warn!("Battery charge limit: {}", err));
            // Then register to dbus server
            ctrl.add_to_server(&mut object_server);
        }
        Err(err) => {
            error!("rog_bios_control: {}", err);
        }
    }

    match CtrlCharge::new(config.clone()) {
        Ok(mut ctrl) => {
            // Do a reload of any settings
            ctrl.reload()
                .unwrap_or_else(|err| warn!("Battery charge limit: {}", err));
            // Then register to dbus server
            ctrl.add_to_server(&mut object_server);
        }
        Err(err) => {
            error!("charge_control: {}", err);
        }
    }

    match CtrlFanAndCpu::new(config.clone()) {
        Ok(mut ctrl) => {
            ctrl.reload()
                .unwrap_or_else(|err| warn!("Profile control: {}", err));
            let tmp = Arc::new(Mutex::new(ctrl));
            FanAndCpuZbus::new(tmp).add_to_server(&mut object_server);
        }
        Err(err) => {
            error!("Profile control: {}", err);
        }
    }

    match CtrlAnime::new(AnimeConfig::load()) {
        Ok(ctrl) => {
            let inner = Arc::new(Mutex::new(ctrl));

            let mut reload = CtrlAnimeReloader(inner.clone());
            reload
                .reload()
                .unwrap_or_else(|err| warn!("AniMe: {}", err));

            let zbus = CtrlAnimeZbus(inner.clone());
            zbus.add_to_server(&mut object_server);

            tasks.push(Box::new(CtrlAnimeTask::new(inner)));
        }
        Err(err) => {
            error!("AniMe control: {}", err);
        }
    }

    let laptop = LaptopLedData::get_data();
    let aura_config = AuraConfig::load(&laptop);
    match CtrlKbdLed::new(laptop, aura_config) {
        Ok(ctrl) => {
            let inner = Arc::new(Mutex::new(ctrl));

            let mut reload = CtrlKbdLedReloader(inner.clone());
            reload
                .reload()
                .unwrap_or_else(|err| warn!("Keyboard LED control: {}", err));

            CtrlKbdLedZbus::new(inner.clone()).add_to_server(&mut object_server);
            let task = CtrlKbdLedTask(inner);
            tasks.push(Box::new(task));
        }
        Err(err) => {
            error!("Keyboard control: {}", err);
        }
    }

    // Graphics switching requires some checks on boot specifically for g-sync capable laptops
    if enable_gfx_switching {
        match CtrlGraphics::new(config.clone()) {
            Ok(mut ctrl) => {
                // Need to check if a laptop has the dedicated gfx switch
                if CtrlRogBios::has_dedicated_gfx_toggle() {
                    if let Ok(ded) = CtrlRogBios::get_gfx_mode() {
                        if let Ok(config) = config.lock() {
                            if ded == 1 {
                                warn!("Dedicated GFX toggle is on but driver mode is not nvidia \nSetting to nvidia driver mode");
                                let devices = ctrl.devices();
                                let bus = ctrl.bus();
                                CtrlGraphics::do_vendor_tasks(
                                    GfxVendors::Nvidia,
                                    false,
                                    &devices,
                                    &bus,
                                )?;
                            } else if ded == 0 {
                                info!("Dedicated GFX toggle is off");
                                let devices = ctrl.devices();
                                let bus = ctrl.bus();
                                CtrlGraphics::do_vendor_tasks(
                                    config.gfx_mode,
                                    false,
                                    &devices,
                                    &bus,
                                )?;
                            }
                        }
                    }
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

    // TODO: implement messaging between threads to check fails

    // Run tasks
    let handle = std::thread::Builder::new()
        .name("asusd watch".to_string())
        .spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(100));

            for ctrl in tasks.iter() {
                ctrl.do_task()
                    .map_err(|err| {
                        warn!("do_task error: {}", err);
                    })
                    .ok();
            }
        });

    // Run zbus server
    object_server
        .with(
            &ObjectPath::from_str_unchecked("/org/asuslinux/Charge"),
            |obj: &CtrlCharge| {
                let x = obj.limit();
                obj.notify_charge(x as u8)
            },
        )
        .map_err(|err| {
            warn!("object_server notify_charge error: {}", err);
        })
        .ok();

    // Loop to check errors and iterate zbus server
    loop {
        if let Err(err) = &handle {
            error!("{}", err);
        }
        if let Err(err) = object_server.try_handle_next() {
            error!("{}", err);
        }
    }
}
