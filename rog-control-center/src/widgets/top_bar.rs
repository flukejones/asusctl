use std::sync::atomic::Ordering;

use crate::RogApp;

impl<'a> RogApp<'a> {
    pub fn top_bar(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self { states, config, .. } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui
                        .checkbox(&mut config.run_in_background, "Run in Background")
                        .clicked()
                        || ui
                            .checkbox(&mut config.startup_in_background, "Startup Hidden")
                            .clicked()
                        || ui
                            .checkbox(&mut config.enable_notifications, "Enable Notifications")
                            .clicked()
                    {
                        states
                            .notifs_enabled
                            .store(config.enable_notifications, Ordering::SeqCst);
                        config
                            .save()
                            .map_err(|err| {
                                states.error = Some(err.to_string());
                            })
                            .ok();
                    }
                });
            });
        });
    }
}
