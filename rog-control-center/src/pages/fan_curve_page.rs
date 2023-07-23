use egui::{RichText, Ui};
use rog_platform::supported::SupportedFunctions;

use crate::system_state::{FanCurvesState, ProfilesState, SystemState};
use crate::widgets::fan_graphs;
use crate::{RogApp, RogDbusClientBlocking};

impl RogApp {
    pub fn fan_curve_page(&mut self, states: &mut SystemState, ctx: &egui::Context) {
        let Self { supported, .. } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Custom fan curves");
            Self::fan_curve(
                supported,
                &mut states.profiles,
                &mut states.fan_curves,
                &states.asus_dbus,
                &mut states.error,
                ui,
            );

            fan_graphs(
                supported,
                &mut states.fan_curves,
                &states.asus_dbus,
                &mut states.error,
                ui,
            );
        });
    }

    fn fan_curve(
        supported: &SupportedFunctions,
        profiles: &mut ProfilesState,
        curves: &mut FanCurvesState,
        dbus: &RogDbusClientBlocking<'_>,
        do_error: &mut Option<String>,
        ui: &mut Ui,
    ) {
        ui.separator();

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Current profile: ");
            ui.label(RichText::new(format!("{}", profiles.current)).strong());
        });

        // ui.horizontal(|ui| {
        //     ui.label("Enabled fan-curves: ");
        //     let mut fan_curve_enable = |profile: Profile, fan: FanCurvePU, mut
        // checked: bool| {         if ui
        //             .add(egui::Checkbox::new(&mut checked, format!("{:?}", fan)))
        //             .changed()
        //         {
        //             dbus.proxies()
        //                 .profile()
        //                 .set_fan_curves_enabled(profile, checked)
        //                 .map_err(|err| {
        //                     *do_error = Some(err.to_string());
        //                 })
        //                 .ok();
        //             changed = true;
        //         }
        //     };

        //     if let Some(curves) = curves.curves.get_mut(&profiles.current) {
        //         for curve in curves.iter_mut() {
        //             fan_curve_enable(profiles.current, curve.fan, curve.enabled);
        //         }
        //     }
        // });

        ui.horizontal(|ui| {
            ui.label("Enabled fan-curves: ");
            let mut checked = false;
            let mut label = String::default();
            if let Some(curves) = curves.curves.get_mut(&profiles.current) {
                for curve in curves.iter() {
                    label.push_str(&<&str>::from(curve.fan).to_ascii_uppercase());
                    label.push(' ');
                    if curve.enabled {
                        // TODO: it's possible to set just one fan to active
                        checked = true;
                    }
                }
            }

            if ui
                .add(egui::Checkbox::new(
                    &mut checked,
                    RichText::new(label).strong(),
                ))
                .changed()
            {
                dbus.proxies()
                    .profile()
                    .set_fan_curves_enabled(profiles.current, checked)
                    .map_err(|err| {
                        *do_error = Some(err.to_string());
                    })
                    .ok();
                changed = true;
            }
        });

        if changed {
            let selected_profile = curves.show_curve;
            let selected_pu = curves.show_graph;

            match FanCurvesState::new(supported, dbus) {
                Ok(f) => *curves = f,
                Err(e) => *do_error = Some(e.to_string()),
            }

            curves.show_curve = selected_profile;
            curves.show_graph = selected_pu;
        }
    }
}
