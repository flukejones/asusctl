use crate::{
    widgets::{anime_power_group, aura_power_group, platform_profile, rog_bios_group},
    RogApp,
};

impl<'a> RogApp<'a> {
    pub fn system_page(&mut self, ctx: &egui::Context) {
        let Self {
            supported,
            states,
            asus_dbus: dbus,
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Experimental application for asusd");

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                let rect = ui.available_rect_before_wrap();
                egui::Grid::new("id_source")
                    .min_col_width(rect.width() / 2.0)
                    .show(ui, |ui| {
                        if supported.platform_profile.platform_profile {
                            ui.vertical(|ui| {
                                ui.separator();
                                platform_profile(states, dbus, ui);
                            });
                        }

                        ui.vertical(|ui| {
                            ui.separator();
                            aura_power_group(supported, states, dbus, ui);
                        });
                        ui.end_row();

                        ui.vertical(|ui| {
                            ui.separator();
                            rog_bios_group(supported, states, dbus, ui);
                        });

                        if supported.anime_ctrl.0 {
                            ui.vertical(|ui| {
                                ui.separator();
                                anime_power_group(supported, states, dbus, ui);
                            });
                        }
                        ui.end_row();
                    });
            });
        });
    }
}
