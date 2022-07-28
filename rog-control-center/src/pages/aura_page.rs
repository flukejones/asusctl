use egui::{Color32, Vec2};
use rog_aura::keys::KeyShape;

use crate::{widgets::aura_modes_group, RogApp};

impl<'a> RogApp<'a> {
    pub fn aura_page(&mut self, ctx: &egui::Context) {
        let Self {
            supported,
            states,
            asus_dbus: dbus,
            ..
        } = self;

        let c = states
            .aura
            .modes
            .get(&states.aura.current_mode)
            .unwrap()
            .colour1;
        let colour = Color32::from_rgb(c.0, c.1, c.2);
        // TODO: animation of colour changes/periods/blending
        egui::CentralPanel::default().show(ctx, |ui| {
            aura_modes_group(supported, states, dbus, ui);

            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            for row in states.keyboard_layout.rows() {
                ui.horizontal(|ui| {
                    for key in row.row() {
                        // your boat
                        let shape = KeyShape::from(key);

                        let label = <&str>::from(key);
                        if shape.is_blank() || shape.is_spacer() {
                            blank(ui, shape);
                        } else if shape.is_group() {
                            key_group(ui, colour, shape.ux(), shape.uy()).on_hover_text(label);
                        } else {
                            key_shape(ui, colour, shape).on_hover_text(label);
                        }
                    }
                });
            }
        });
    }
}

fn key_shape(ui: &mut egui::Ui, colour: Color32, shape: KeyShape) -> egui::Response {
    let desired_size =
        ui.spacing().interact_size.y * egui::vec2(2.0 * shape.ux(), 2.0 * shape.uy());
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

fn blank(ui: &mut egui::Ui, shape: KeyShape) {
    let desired_size =
        ui.spacing().interact_size.y * egui::vec2(2.0 * shape.ux(), 2.0 * shape.uy());
    ui.allocate_exact_size(desired_size, egui::Sense::click());
}
