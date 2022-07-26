use std::sync::atomic::Ordering;

use egui::{vec2, Align2, Button, FontId, Id, Rect, RichText, Sense, Vec2};

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

                ui.horizontal(|ui| {
                    egui::global_dark_light_mode_switch(ui);
                    egui::warn_if_debug_build(ui);
                });

                /***********************************************************/
                // Drag area
                let text_color = ctx.style().visuals.text_color();
                let mut titlebar_rect = ui.available_rect_before_wrap();
                titlebar_rect.max.x -= titlebar_rect.height();
                if ui
                    .interact(titlebar_rect, Id::new("title_bar"), Sense::drag())
                    .drag_started()
                {
                    frame.drag_window();
                }
                /***********************************************************/
                let height = titlebar_rect.height();
                // Paint the title:
                ui.painter().text(
                    titlebar_rect.center_top() + vec2(0.0, height / 2.0),
                    Align2::CENTER_CENTER,
                    "ROG Control Center",
                    FontId::proportional(height - 2.0),
                    text_color,
                );
                // Add the close button:
                let close_response = ui.put(
                    Rect::from_min_size(titlebar_rect.right_top(), Vec2::splat(height)),
                    Button::new(RichText::new("❌").size(height - 4.0)).frame(false),
                );
                if close_response.clicked() {
                    frame.quit();
                }
            });
        });
    }
}
