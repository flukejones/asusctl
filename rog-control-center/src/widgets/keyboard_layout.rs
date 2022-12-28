use egui::{Color32, Pos2};
use rog_aura::advanced::LedCode;
use rog_aura::layouts::{KeyLayout, KeyShape};
use rog_aura::{AdvancedAuraType, AuraModeNum};

use crate::system_state::AuraState;

const SCALE: f32 = 2.0;

// TODO:
// - Multizone: draw regions? While iterating keys check if located in one of
//   the 4 regions and mark
// - Tab for advanced effects
// - Keys need to select colour themselves

pub fn keyboard(
    ui: &mut egui::Ui,
    keyboard_layout: &KeyLayout,
    states: &mut AuraState,
    colour: Color32,
) {
    let (keyboard_is_multizoned, keyboard_width, keyboard_is_per_key) =
        match keyboard_layout.advanced_type() {
            AdvancedAuraType::PerKey => (false, 0.0, true),
            AdvancedAuraType::None => (false, keyboard_layout.max_width(), false),
            AdvancedAuraType::Zoned(zones) => {
                let width = if let Some(row) = keyboard_layout.rows_ref().get(2) {
                    row.width() as f32
                } else {
                    0.0
                };
                (!zones.contains(&LedCode::SingleZone), width, false)
            }
        };
    let mut start_pos = None;

    let y = ui.spacing().interact_size.y;
    let this_size = ui.available_size();
    let keys_width = keyboard_layout.max_width() * SCALE * y;
    let keys_height = keyboard_layout.max_height() * SCALE * y;
    let keyboard_height = keyboard_layout.keyboard_height() * SCALE;
    let x_start = (this_size.x - keys_width) / SCALE;
    let y_start = (this_size.y - keys_height) / SCALE;

    // Initial colour states
    let mut input_colour = colour;
    let mut key_colour = colour;
    if states.current_mode == AuraModeNum::Rainbow && !keyboard_is_per_key {
        key_colour = Color32::from_rgb(
            (states.wave_red[0] as u32 * 255 / 100) as u8,
            (states.wave_green[0] as u32 * 255 / 100) as u8,
            (states.wave_blue[0] as u32 * 255 / 100) as u8,
        );
    }

    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
    blank(ui, 0.0, y_start / y);

    for row in keyboard_layout.rows() {
        ui.horizontal_top(|ui| {
            blank(ui, x_start / y, 0.0);
            for (i, key) in row.row().enumerate() {
                // For per-key rainbow which cascades across
                if states.current_mode == AuraModeNum::Rainbow && keyboard_is_per_key
                    || key.0.is_lightbar_zone()
                {
                    key_colour = Color32::from_rgb(
                        (states.wave_red[i] as u32 * 255 / 100) as u8,
                        (states.wave_green[i] as u32 * 255 / 100) as u8,
                        (states.wave_blue[i] as u32 * 255 / 100) as u8,
                    );
                }

                if (keyboard_is_multizoned && !key.0.is_lightbar_zone())
                    && states.current_mode == AuraModeNum::Rainbow
                {
                    input_colour = key_colour;
                    key_colour = Color32::TRANSPARENT;
                }

                let label = <&str>::from(key.0);
                let mut shape = key.1.clone();
                shape.scale(SCALE);

                match shape {
                    KeyShape::Led {
                        width,
                        height,
                        pad_left,
                        pad_right,
                        pad_top,
                        pad_bottom,
                    } => {
                        let (pos, response) = key_shape(
                            ui, key_colour, width, height, pad_left, pad_right, pad_top, pad_bottom,
                        );
                        if start_pos.is_none() {
                            start_pos = Some(pos);
                        } else if let Some(old_pos) = start_pos.as_mut() {
                            if !key.0.is_lightbar_zone() {
                                if pos.x < old_pos.x {
                                    old_pos.x = pos.x;
                                }
                                if pos.y < old_pos.y {
                                    old_pos.y = pos.y;
                                }
                            }
                        }
                        if response.on_hover_text(label).clicked() && keyboard_is_per_key {
                            // TODO: set an effect on the LedCode
                        }
                    }
                    KeyShape::Blank { width, height } => {
                        blank(ui, width, height);
                    }
                }
            }
        });
    }

    if keyboard_is_multizoned {
        let zone_width = keyboard_width * SCALE / 4.0 - 0.1;
        for n in 0..4 {
            if states.current_mode == AuraModeNum::Rainbow {
                input_colour = Color32::from_rgba_unmultiplied(
                    (states.wave_red[n] as u32 * 255 / 100) as u8,
                    (states.wave_green[n] as u32 * 255 / 100) as u8,
                    (states.wave_blue[n] as u32 * 255 / 100) as u8,
                    70,
                );
            }
            if let Some(mut pos) = start_pos {
                pos.x += n as f32 * zone_width * y;
                let response = zone_shape(ui, input_colour, pos, zone_width, keyboard_height);
                let label = format!("Zone {}", 1 + n);
                if response.on_hover_text(label).clicked() {
                    // TODO: set an effect on the zone
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn key_shape(
    ui: &mut egui::Ui,
    colour: Color32,
    width: f32,
    height: f32,
    pad_left: f32,
    pad_right: f32,
    pad_top: f32,
    pad_bottom: f32,
) -> (egui::Pos2, egui::Response) {
    // First, get some space
    let y = ui.spacing().interact_size.y;
    let desired_size = y * egui::vec2(width + pad_left + pad_right, height + pad_top + pad_bottom);
    let (mut rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    // rect = rect.shrink(3.0);
    if response.clicked() {
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, response.clicked(), "")
    });

    if ui.is_rect_visible(rect) {
        // Now set the actual visible rect
        let visuals = ui.style().interact_selectable(&response, true);
        let size = y * egui::vec2(width, height);
        rect.set_width(size.x);
        rect.set_height(size.y);
        let center = Pos2::new(
            rect.center().x + pad_left * y,
            rect.center().y + pad_top * y,
        );
        rect.set_center(center);
        // let rect = rect.expand(visuals.expansion);
        ui.painter().rect(rect, 0.1, colour, visuals.fg_stroke);
    }

    (rect.left_top(), response)
}

#[allow(clippy::too_many_arguments)]
fn zone_shape(
    ui: &mut egui::Ui,
    mut colour: Color32,
    pos: Pos2,
    width: f32,
    height: f32,
) -> egui::Response {
    // First, get some space
    let y = ui.spacing().interact_size.y;
    let desired_size = y * egui::vec2(width, height);
    let rect = egui::Rect::from_min_size(pos, desired_size);
    let mut response = ui.allocate_rect(rect, egui::Sense::click());
    // rect = rect.shrink(3.0);
    if response.clicked() {
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, response.clicked(), "")
    });

    if ui.is_rect_visible(rect) {
        // Now set the actual visible rect
        let visuals = ui.style().interact_selectable(&response, true);
        // let rect = rect.expand(visuals.expansion);
        colour[3] = 20;
        ui.painter().rect(rect, 0.1, colour, visuals.fg_stroke);
    }

    response
}

fn blank(ui: &mut egui::Ui, ux: f32, uy: f32) {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(ux, uy);
    ui.allocate_exact_size(desired_size, egui::Sense::click());
}
