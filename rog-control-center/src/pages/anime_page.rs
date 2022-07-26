use crate::RogApp;

impl<'a> RogApp<'a> {
    pub fn anime_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("In progress");
        });
    }
}
