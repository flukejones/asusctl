use egui::plot::{Line, Plot, Points};
use egui::Ui;
use rog_platform::platform::PlatformPolicy;
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::FanCurvePU;

use crate::system_state::FanCurvesState;
use crate::RogDbusClientBlocking;

pub fn fan_graphs(
    curves: &mut FanCurvesState,
    dbus: &RogDbusClientBlocking<'_>,
    do_error: &mut Option<String>,
    ui: &mut Ui,
) {
    ui.separator();

    let mut item = |profile: PlatformPolicy, ui: &mut Ui| {
        ui.group(|ui| {
            if ui
                .selectable_value(&mut curves.show_curve, profile, format!("{profile:?}"))
                .clicked()
            {
                dbus.proxies()
                    .platform()
                    .set_throttle_thermal_policy(profile)
                    .ok();
            }
            ui.add_enabled_ui(curves.show_curve == profile, |ui| {
                if curves.available_fans.contains(&FanCurvePU::CPU) {
                    ui.selectable_value(
                        &mut curves.show_graph,
                        FanCurvePU::CPU,
                        format!("{:?}", FanCurvePU::CPU),
                    );
                }
                if curves.available_fans.contains(&FanCurvePU::GPU) {
                    ui.selectable_value(
                        &mut curves.show_graph,
                        FanCurvePU::GPU,
                        format!("{:?}", FanCurvePU::GPU),
                    );
                }
                if curves.available_fans.contains(&FanCurvePU::MID) {
                    ui.selectable_value(
                        &mut curves.show_graph,
                        FanCurvePU::MID,
                        format!("{:?}", FanCurvePU::MID),
                    );
                }
            });
        });
    };

    ui.horizontal_wrapped(|ui| {
        for a in &curves.curves {
            item(*a.0, ui);
        }
    });

    let curve = curves.curves.get_mut(&curves.show_curve).unwrap();

    let mut data = &mut CurveData::default();
    for c in curve {
        if c.fan == curves.show_graph {
            data = c;
            break;
        }
    }

    let mut points: Vec<[f64; 2]> = data
        .temp
        .iter()
        .enumerate()
        .map(|(idx, x)| {
            let x = *x as f64;
            let y = ((data.pwm[idx] as u32) * 100 / 255) as f64;
            [x, y]
        })
        .collect();

    for i in 0..points.len() - 1 {
        if i > 0 && i < points.len() - 1 {
            if points[i][0] < points[i - 1][0] {
                points[i][0] = points[i - 1][0] + 1.0;
                data.temp[i] = points[i - 1][0] as u8;
            }
            if points[i][0] >= points[i + 1][0] {
                points[i + 1][0] = points[i][0] + 1.0;
                data.temp[i + 1] = points[i][0] as u8;
            }
            if points[i][1] < points[i - 1][1] {
                points[i][1] = points[i - 1][1] + 1.0;
                data.pwm[i] = (points[i - 1][1] * 255.0 / 100.0 + 1.0).floor() as u8;
            }
            if points[i][1] >= points[i + 1][1] {
                points[i + 1][1] = points[i][1] + 1.0;
                data.pwm[i + 1] = (points[i][1] * 255.0 / 100.0 + 1.0).floor() as u8;
            }
        }
    }

    let line = Line::new(points.clone()).width(2.0);
    let points = Points::new(points).radius(3.0);

    Plot::new("fan_curves")
        .view_aspect(1.666)
        // .center_x_axis(true)
        // .center_y_axis(true)
        .include_x(0.0)
        .include_x(104.0)
        .include_y(0.0)
        .include_y(106.0)
        .allow_scroll(false)
        .allow_drag(false)
        .allow_boxed_zoom(false)
        // .x_axis_formatter(|d, _r| format!("{}", d))
        // .y_axis_formatter(|d, _r| format!("{:.*}%", 1, d))
        .label_formatter(|name, value| {
            if !name.is_empty() {
                format!("{}: {:.*}%", name, 1, value.y)
            } else {
                format!("Temp {}c\nFan {:.*}%", value.x as u8, 1, value.y)
            }
        })
        .show(ui, |plot_ui| {
            if plot_ui.plot_hovered() {
                let mut idx = 0;

                if let Some(point) = plot_ui.pointer_coordinate() {
                    let mut x: i32 = point.x as i32;
                    for (i, n) in data.temp.iter().enumerate() {
                        let tmp = x.min((point.x as i32 - *n as i32).abs());
                        if tmp < x {
                            x = tmp;
                            idx = i;
                        }
                    }

                    if plot_ui.plot_clicked() {
                        data.temp[idx] = point.x as u8;
                        data.pwm[idx] = (point.y * 255.0 / 100.0) as u8;
                    } else {
                        let drag = plot_ui.pointer_coordinate_drag_delta();
                        if drag.length_sq() != 0.0 {
                            data.temp[idx] = (point.x as f32 + drag.x) as u8;
                            data.pwm[idx] = ((point.y as f32 + drag.y) * 255.0 / 100.0) as u8;
                        }
                    }
                }
            }
            plot_ui.line(line);
            plot_ui.points(points);
        });

    let mut set = false;
    let mut clear = false;
    let mut reset = false;
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        set = ui.add(egui::Button::new("Apply Profile")).clicked();
        clear = ui.add(egui::Button::new("Clear Profile Changes")).clicked();
        reset = ui.add(egui::Button::new("Factory Reset Profile")).clicked();
    });

    if set {
        dbus.proxies()
            .fan_curves()
            .set_fan_curve(curves.show_curve, data.clone())
            .map_err(|err| {
                *do_error = Some(err.to_string());
            })
            .ok();
    }

    if clear {
        if let Ok(curve) = dbus
            .proxies()
            .fan_curves()
            .fan_curve_data(curves.show_curve)
            .map_err(|err| {
                *do_error = Some(err.to_string());
            })
        {
            if let Some(value) = curves.curves.get_mut(&curves.show_curve) {
                *value = curve;
            }
        }
    }

    if reset {
        dbus.proxies()
            .fan_curves()
            .reset_profile_curves(curves.show_curve)
            .map_err(|err| {
                *do_error = Some(err.to_string());
            })
            .ok();

        match FanCurvesState::new(dbus) {
            Ok(f) => *curves = f,
            Err(e) => *do_error = Some(e.to_string()),
        }
    }
}
