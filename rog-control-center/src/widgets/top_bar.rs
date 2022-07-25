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
                    let (mut in_bg, mut hidden) =
                        { (config.run_in_background, config.startup_in_background) };
                    if ui.checkbox(&mut in_bg, "Run in Background").clicked() {
                        config.run_in_background = in_bg;
                        config
                            .save()
                            .map_err(|err| {
                                states.error = Some(err.to_string());
                            })
                            .ok();
                    }
                    if ui.checkbox(&mut hidden, "Startup Hidden").clicked() {
                        config.startup_in_background = in_bg;
                        config
                            .save()
                            .map_err(|err| {
                                states.error = Some(err.to_string());
                            })
                            .ok();
                    }
                    if ui
                        .checkbox(&mut config.enable_notifications, "Enable Notifications")
                        .clicked()
                    {
                        config.enable_notifications = in_bg;
                        // TODO: set an atomicbool used in the notif thread
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
