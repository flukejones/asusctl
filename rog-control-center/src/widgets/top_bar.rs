use egui::{vec2, Align2, FontId, Id, Sense};

use crate::{RogApp, VERSION};

impl RogApp {
    pub fn top_bar(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.horizontal(|ui| {
                    egui::global_dark_light_mode_buttons(ui);
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
                    titlebar_rect.right_top() + vec2(0.0, height / 2.0),
                    Align2::RIGHT_CENTER,
                    format!("v{}", VERSION),
                    FontId::proportional(height - 2.0),
                    text_color,
                );
                // // Add the close button:
                // let close_response = ui.put(
                //     Rect::from_min_size(titlebar_rect.right_top(), Vec2::splat(height)),
                //     Button::new(RichText::new("❌").size(height - 4.0)).frame(false),
                // );
                // if close_response.clicked() {
                //     frame.close();
                // }
            });
        });
    }
}
