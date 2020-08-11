use daemon::{
    config::Config, ctrl_anime::CtrlAnimeDisplay, ctrl_charge::CtrlCharge,
    ctrl_fan_cpu::CtrlFanAndCPU, ctrl_leds::CtrlKbdBacklight, dbus::dbus_create_tree,
    laptops::match_laptop,
};

use dbus::{channel::Sender, nonblock::SyncConnection, tree::Signal};

use asus_nb::{DBUS_IFACE, DBUS_NAME, DBUS_PATH};
use daemon::Controller;
use dbus_tokio::connection;
use log::LevelFilter;
use log::{error, info, warn};
use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = env_logger::Builder::new();
    logger
        .target(env_logger::Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .init();

    info!("Version: {}", daemon::VERSION);
    start_daemon().await?;
    Ok(())
}

// Timing is such that:
// - interrupt write is minimum 1ms (sometimes lower)
// - read interrupt must timeout, minimum of 1ms
// - for a single usb packet, 2ms total.
// - to maintain constant times of 1ms, per-key colours should use
//   the effect endpoint so that the complete colour block is written
//   as fast as 1ms per row of the matrix inside it. (10ms total time)
//
// DBUS processing takes 6ms if not tokiod
pub async fn start_daemon() -> Result<(), Box<dyn Error>> {
    let laptop = match_laptop();
    let mut config = if let Some(laptop) = laptop.as_ref() {
        Config::default().load(laptop.supported_modes())
    } else {
        Config::default().load(&[])
    };

    let mut led_control = if let Some(laptop) = laptop {
        CtrlKbdBacklight::new(laptop.usb_product(), laptop.supported_modes().to_owned())
            .map_or_else(
                |err| {
                    error!("{}", err);
                    None
                },
                Some,
            )
    } else {
        None
    };

    let mut charge_control = CtrlCharge::new().map_or_else(
        |err| {
            error!("{}", err);
            None
        },
        Some,
    );

    let mut fan_control = CtrlFanAndCPU::new().map_or_else(
        |err| {
            error!("{}", err);
            None
        },
        Some,
    );

    // Reload settings
    if let Some(ctrl) = fan_control.as_mut() {
        ctrl.reload_from_config(&mut config)
            .await
            .unwrap_or_else(|err| warn!("Fan mode: {}", err));
    }

    if let Some(ctrl) = charge_control.as_mut() {
        ctrl.reload_from_config(&mut config)
            .await
            .unwrap_or_else(|err| warn!("Battery charge limit: {}", err));
    }

    if let Some(ctrl) = led_control.as_mut() {
        ctrl.reload_from_config(&mut config)
            .await
            .unwrap_or_else(|err| warn!("Reload settings: {}", err));
    }

    let (resource, connection) = connection::new_system_sync()?;
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    connection
        .request_name(DBUS_NAME, false, true, true)
        .await?;

    let config = Arc::new(Mutex::new(config));
    let (
        tree,
        aura_command_recv,
        animatrix_recv,
        fan_mode_recv,
        charge_limit_recv,
        led_changed_signal,
        fanmode_signal,
        charge_limit_signal,
    ) = dbus_create_tree(config.clone());

    // We add the tree to the connection so that incoming method calls will be handled.
    tree.start_receive_send(&*connection);

    // Send boot signals
    send_boot_signals(
        connection.clone(),
        config.clone(),
        fanmode_signal.clone(),
        charge_limit_signal.clone(),
        led_changed_signal.clone(),
    )
    .await?;

    // For helping with processing signals
    start_signal_task(
        connection.clone(),
        config.clone(),
        fanmode_signal,
        charge_limit_signal,
    );

    // Begin all tasks
    let mut handles = Vec::new();
    if let Ok(ctrl) = CtrlAnimeDisplay::new() {
        handles.append(&mut ctrl.spawn_task_loop(config.clone(), animatrix_recv, None, None));
    }

    if let Some(ctrl) = fan_control.take() {
        handles.append(&mut ctrl.spawn_task_loop(config.clone(), fan_mode_recv, None, None));
    }

    if let Some(ctrl) = charge_control.take() {
        handles.append(&mut ctrl.spawn_task_loop(config.clone(), charge_limit_recv, None, None));
    }

    if let Some(ctrl) = led_control.take() {
        handles.append(&mut ctrl.spawn_task_loop(
            config.clone(),
            aura_command_recv,
            Some(connection.clone()),
            Some(led_changed_signal),
        ));
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

// TODO: Move these in to the controllers tasks
fn start_signal_task(
    connection: Arc<SyncConnection>,
    config: Arc<Mutex<Config>>,
    fanmode_signal: Arc<Signal<()>>,
    charge_limit_signal: Arc<Signal<()>>,
) {
    tokio::spawn(async move {
        // Some small things we need to track, without passing all sorts of stuff around
        let mut last_fan_mode = config.lock().await.power_profile;
        let mut last_charge_limit = config.lock().await.bat_charge_limit;
        loop {
            // Use tokio sleep to not hold up other threads
            tokio::time::delay_for(std::time::Duration::from_millis(500)).await;

            let config = config.lock().await;
            if config.power_profile != last_fan_mode {
                last_fan_mode = config.power_profile;
                connection
                    .send(
                        fanmode_signal
                            .msg(&DBUS_PATH.into(), &DBUS_IFACE.into())
                            .append1(last_fan_mode),
                    )
                    .unwrap_or_else(|_| 0);
            }

            if config.bat_charge_limit != last_charge_limit {
                last_charge_limit = config.bat_charge_limit;
                connection
                    .send(
                        charge_limit_signal
                            .msg(&DBUS_PATH.into(), &DBUS_IFACE.into())
                            .append1(last_charge_limit),
                    )
                    .unwrap_or_else(|_| 0);
            }
        }
    });
}

async fn send_boot_signals(
    connection: Arc<SyncConnection>,
    config: Arc<Mutex<Config>>,
    fanmode_signal: Arc<Signal<()>>,
    charge_limit_signal: Arc<Signal<()>>,
    led_changed_signal: Arc<Signal<()>>,
) -> Result<(), Box<dyn Error>> {
    let config = config.lock().await;

    if let Some(data) = config.get_led_mode_data(config.kbd_backlight_mode) {
        connection
            .send(
                led_changed_signal
                    .msg(&DBUS_PATH.into(), &DBUS_IFACE.into())
                    .append1(serde_json::to_string(data)?),
            )
            .unwrap_or_else(|_| 0);
    }

    connection
        .send(
            fanmode_signal
                .msg(&DBUS_PATH.into(), &DBUS_IFACE.into())
                .append1(config.power_profile),
        )
        .unwrap_or_else(|_| 0);

    connection
        .send(
            charge_limit_signal
                .msg(&DBUS_PATH.into(), &DBUS_IFACE.into())
                .append1(config.bat_charge_limit),
        )
        .unwrap_or_else(|_| 0);

    Ok(())
}
