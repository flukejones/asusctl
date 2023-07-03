use crate::system_state::SystemState;
use crate::widgets::app_settings;
use crate::RogApp;

impl RogApp {
    pub fn app_settings_page(&mut self, states: &mut SystemState, ctx: &egui::Context) {
        let Self { config, .. } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                let rect = ui.available_rect_before_wrap();
                egui::Grid::new("grid_of_bits")
                    .min_col_width(rect.width() / 2.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.separator();
                            app_settings(config, states, ui);
                        });
                    });
            })
        });
    }
}
