use crate::RogApp;

impl RogApp {
    pub fn anime_page(&mut self, ctx: &eframe::egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("In progress");
        });
    }
}
