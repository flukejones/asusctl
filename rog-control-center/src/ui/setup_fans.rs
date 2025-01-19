use std::sync::{Arc, Mutex};

use log::{error, info};
use rog_dbus::zbus_fan_curves::FanCurvesProxy;
use rog_platform::platform::PlatformProfile;
use rog_profiles::fan_curve_set::CurveData;
use slint::{ComponentHandle, Model, Weak};

use crate::config::Config;
use crate::{FanPageData, FanType, MainWindow, Node};

pub fn update_fan_data(
    handle: Weak<MainWindow>,
    bal: Vec<CurveData>,
    perf: Vec<CurveData>,
    quiet: Vec<CurveData>
) {
    handle
        .upgrade_in_event_loop(move |handle| {
            let global = handle.global::<FanPageData>();
            let collect = |temp: &[u8], pwm: &[u8]| -> slint::ModelRc<Node> {
                let tmp: Vec<Node> = temp
                    .iter()
                    .zip(pwm.iter())
                    .map(|(x, y)| Node {
                        x: *x as f32,
                        y: *y as f32
                    })
                    .collect();
                tmp.as_slice().into()
            };

            for fan in bal {
                global.set_balanced_available(true);
                match fan.fan {
                    rog_profiles::FanCurvePU::CPU => {
                        global.set_cpu_fan_available(true);
                        global.set_balanced_cpu_enabled(fan.enabled);
                        global.set_balanced_cpu(collect(&fan.temp, &fan.pwm))
                    }
                    rog_profiles::FanCurvePU::GPU => {
                        global.set_gpu_fan_available(true);
                        global.set_balanced_gpu_enabled(fan.enabled);
                        global.set_balanced_gpu(collect(&fan.temp, &fan.pwm))
                    }
                    rog_profiles::FanCurvePU::MID => {
                        global.set_mid_fan_available(true);
                        global.set_balanced_mid_enabled(fan.enabled);
                        global.set_balanced_mid(collect(&fan.temp, &fan.pwm))
                    }
                }
            }
            for fan in perf {
                global.set_performance_available(true);
                match fan.fan {
                    rog_profiles::FanCurvePU::CPU => {
                        global.set_performance_cpu_enabled(fan.enabled);
                        global.set_performance_cpu(collect(&fan.temp, &fan.pwm))
                    }
                    rog_profiles::FanCurvePU::GPU => {
                        global.set_performance_gpu_enabled(fan.enabled);
                        global.set_performance_gpu(collect(&fan.temp, &fan.pwm))
                    }
                    rog_profiles::FanCurvePU::MID => {
                        global.set_performance_mid_enabled(fan.enabled);
                        global.set_performance_mid(collect(&fan.temp, &fan.pwm))
                    }
                }
            }
            for fan in quiet {
                global.set_quiet_available(true);
                match fan.fan {
                    rog_profiles::FanCurvePU::CPU => {
                        global.set_quiet_cpu(collect(&fan.temp, &fan.pwm))
                    }
                    rog_profiles::FanCurvePU::GPU => {
                        global.set_quiet_gpu(collect(&fan.temp, &fan.pwm))
                    }
                    rog_profiles::FanCurvePU::MID => {
                        global.set_quiet_mid(collect(&fan.temp, &fan.pwm))
                    }
                }
            }
        })
        .map_err(|e| error!("update_fan_data: upgrade_in_event_loop: {e:?}"))
        .ok();
}

pub fn setup_fan_curve_page(ui: &MainWindow, _config: Arc<Mutex<Config>>) {
    let handle = ui.as_weak();

    tokio::spawn(async move {
        // Create the connections/proxies here to prevent future delays in process
        let conn = if let Ok(conn) = zbus::Connection::system().await.map_err(|e| error!("{e:}")) {
            conn
        } else {
            return;
        };
        let fans = if let Ok(fans) = FanCurvesProxy::new(&conn).await.map_err(|e| error!("{e:}")) {
            fans
        } else {
            info!(
                "This device may not have an Fan Curve control. If not then the error can be \
                 ignored"
            );
            return;
        };

        let handle_copy = handle.clone();
        // Do initial setup
        let Ok(balanced) = fans
            .fan_curve_data(PlatformProfile::Balanced)
            .await
            .map_err(|e| error!("{e:}"))
        else {
            return;
        };
        let Ok(perf) = fans
            .fan_curve_data(PlatformProfile::Performance)
            .await
            .map_err(|e| error!("{e:}"))
        else {
            return;
        };
        let Ok(quiet) = fans
            .fan_curve_data(PlatformProfile::Quiet)
            .await
            .map_err(|e| error!("{e:}"))
        else {
            return;
        };
        update_fan_data(handle, balanced, perf, quiet);

        let handle_next1 = handle_copy.clone();
        handle_copy
            .upgrade_in_event_loop(move |handle| {
                let global = handle.global::<FanPageData>();
                let fans1 = fans.clone();
                global.on_set_profile_default(move |profile| {
                    let fans = fans1.clone();
                    let handle_next = handle_next1.clone();
                    tokio::spawn(async move {
                        if fans.set_curves_to_defaults(profile.into()).await.is_err() {
                            return;
                        }
                        let Ok(balanced) = fans
                            .fan_curve_data(PlatformProfile::Balanced)
                            .await
                            .map_err(|e| error!("{e:}"))
                        else {
                            return;
                        };
                        let Ok(perf) = fans
                            .fan_curve_data(PlatformProfile::Performance)
                            .await
                            .map_err(|e| error!("{e:}"))
                        else {
                            return;
                        };
                        let Ok(quiet) = fans
                            .fan_curve_data(PlatformProfile::Quiet)
                            .await
                            .map_err(|e| error!("{e:}"))
                        else {
                            return;
                        };
                        update_fan_data(handle_next, balanced, perf, quiet);
                    });
                });
                global.on_set_fan_data(move |fan, profile, enabled, data| {
                    let fans = fans.clone();
                    let data: Vec<Node> = data.iter().collect();
                    let data = fan_data_for(fan, enabled, data);
                    tokio::spawn(async move {
                        fans.set_fan_curve(profile.into(), data)
                            .await
                            .map_err(|e| error!("{e:}"))
                            .ok()
                    });
                });
            })
            .map_err(|e| error!("setup_fan_curve_page: upgrade_in_event_loop: {e:?}"))
            .ok();
    });
}

fn fan_data_for(fan: FanType, enabled: bool, data: Vec<Node>) -> CurveData {
    let mut temp = [0u8; 8];
    let mut pwm = [0u8; 8];
    for (i, n) in data.iter().enumerate() {
        if i == 8 {
            break;
        }
        temp[i] = n.x as u8;
        pwm[i] = n.y as u8;
    }

    CurveData {
        fan: fan.into(),
        pwm,
        temp,
        enabled
    }
}
