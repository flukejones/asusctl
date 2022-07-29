use egui::{Align, Color32, Vec2};
use rog_aura::{keys::KeyShape, layouts::KeyLayout};

pub fn keyboard(ui: &mut egui::Ui, keyboard_layout: &KeyLayout, colour: Color32) {
    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
    let mut arrows_done = false;
    let mut rog_done = false;
    for row in keyboard_layout.rows() {
        ui.horizontal_top(|ui| {
            for key in row.row() {
                // your boat
                let height = if rog_done {
                    row.height()
                } else {
                    // Use the first item (always a blank) to stand off the row
                    rog_done = true;
                    1.2
                };
                let shape = KeyShape::from(key);

                let label = <&str>::from(key);
                if shape.is_arrow_cluster() {
                    if !arrows_done {
                        arrow_cluster(ui, colour);
                        arrows_done = true;
                    }
                } else if shape.is_blank() || shape.is_spacer() {
                    blank(ui, shape.width(), height);
                } else if shape.is_group() {
                    key_group(ui, colour, shape.width(), height).on_hover_text(label);
                } else {
                    key_shape(ui, colour, shape.width(), height).on_hover_text(label);
                }
            }
        });
    }
}

fn key_shape(ui: &mut egui::Ui, colour: Color32, ux: f32, uy: f32) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0 * ux, 2.0 * uy);
    let (mut rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    rect = rect.shrink(3.0);
    if response.clicked() {
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, response.clicked(), "")
    });

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, true);
        let rect = rect.expand(visuals.expansion);
        ui.painter().rect(rect, 0.1, colour, visuals.fg_stroke);
    }

    response
}

fn key_group(ui: &mut egui::Ui, colour: Color32, ux: f32, uy: f32) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0 * ux, 2.0 * uy);
    let (mut rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    rect = rect.shrink2(Vec2::new(3.0, 3.0));
    if response.clicked() {
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, response.clicked(), "")
    });

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, true);
        let rect = rect.expand(visuals.expansion);
        let mut stroke = visuals.fg_stroke;
        stroke.color = visuals.bg_fill;
        ui.painter().rect(rect, 0.1, colour, stroke);
    }

    response
}

fn blank(ui: &mut egui::Ui, ux: f32, uy: f32) {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0 * ux, 2.0 * uy);
    ui.allocate_exact_size(desired_size, egui::Sense::click());
}

/// Draws entire arrow cluster block. This is visibly different to the split-arrows.
fn arrow_cluster(ui: &mut egui::Ui, colour: Color32) {
    let height = 0.7;
    let space = KeyShape::ArrowSpacer;
    let shape = KeyShape::Arrow;
    ui.horizontal_top(|ui| {
        ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
            blank(ui, space.width(), height);
            ui.horizontal(|ui| {
                blank(ui, KeyShape::RowEndSpacer.width(), height);
                blank(ui, KeyShape::RowEndSpacer.width(), height);
                key_shape(ui, colour, shape.width(), height).on_hover_text("Left");
            });
        });
        ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
            key_shape(ui, colour, shape.width(), height).on_hover_text("Up");
            key_shape(ui, colour, shape.width(), height).on_hover_text("Down");
        });
        ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
            blank(ui, space.width(), height);
            key_shape(ui, colour, shape.width(), height).on_hover_text("Right");
        });
    });
}
