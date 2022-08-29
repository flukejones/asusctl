use crate::{
    page_states::{FanCurvesState, ProfilesState},
    widgets::fan_graphs,
    RogApp, RogDbusClientBlocking,
};
use egui::Ui;
use rog_platform::supported::SupportedFunctions;
use rog_profiles::Profile;

impl<'a> RogApp<'a> {
    pub fn fan_curve_page(&mut self, ctx: &egui::Context) {
        let Self {
            supported,
            states,
            asus_dbus: dbus,
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Custom fan curves");
            ui.label("A fan curve is only active when the related profile is active and the curve is enabled");
            Self::fan_curve(
                supported,
                &mut states.profiles,
                &mut states.fan_curves,
                dbus, &mut states.error,
                ui,
            );

            fan_graphs(supported, &mut states.profiles, &mut states.fan_curves, dbus, &mut states.error, ui);
        });
    }

    fn fan_curve(
        supported: &SupportedFunctions,
        profiles: &mut ProfilesState,
        curves: &mut FanCurvesState,
        dbus: &RogDbusClientBlocking,
        do_error: &mut Option<String>,
        ui: &mut Ui,
    ) {
        ui.separator();
        ui.label("Enabled fan-curves");

        let mut changed = false;
        ui.horizontal(|ui| {
            let mut item = |p: Profile, curves: &mut FanCurvesState, mut checked: bool| {
                if ui
                    .add(egui::Checkbox::new(&mut checked, format!("{:?}", p)))
                    .changed()
                {
                    dbus.proxies()
                        .profile()
                        .set_fan_curve_enabled(p, checked)
                        .map_err(|err| {
                            *do_error = Some(err.to_string());
                        })
                        .ok();

                    if !checked {
                        curves.enabled.remove(&p);
                    } else {
                        curves.enabled.insert(p);
                    }
                    changed = true;
                }
            };

            profiles.list.sort();
            for f in profiles.list.iter() {
                item(*f, curves, curves.enabled.contains(f));
            }
        });

        if changed {
            let selected_profile = curves.show_curve;
            let selected_PU = curves.show_graph;

            let notif = curves.was_notified.clone();
            match FanCurvesState::new(notif, supported, dbus) {
                Ok(f) => *curves = f,
                Err(e) => *do_error = Some(e.to_string()),
            }

            curves.show_curve = selected_profile;
            curves.show_graph = selected_PU;
        }
    }
}
