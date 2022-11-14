use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use egui::{RichText, Ui};
use rog_aura::{AuraEffect, AuraModeNum, AuraZone, Colour, Speed};
use rog_platform::supported::SupportedFunctions;

use crate::page_states::{AuraState, PageDataStates};

pub fn aura_modes_group(
    supported: &SupportedFunctions,
    states: &mut PageDataStates,
    freq: &mut Arc<AtomicU8>,
    ui: &mut Ui,
) {
    let mut changed = false;
    let mut selected = states.aura.current_mode;
    let allowed = AuraEffect::allowed_parameters(selected);

    let has_keyzones = supported
        .keyboard_led
        .multizone_led_mode
        .contains(&AuraZone::Key2);
    let has_logo = supported
        .keyboard_led
        .multizone_led_mode
        .contains(&AuraZone::Logo);
    let has_lightbar = supported
        .keyboard_led
        .multizone_led_mode
        .contains(&AuraZone::BarLeft)
        || supported
            .keyboard_led
            .multizone_led_mode
            .contains(&AuraZone::BarRight);

    ui.heading("Aura modes");
    let mut item = |a: AuraModeNum, ui: &mut Ui| {
        if ui
            .selectable_value(&mut selected, a, format!("{:?}", a))
            .clicked()
        {
            changed = true;
        }
    };

    ui.horizontal_wrapped(|ui| {
        for a in states.aura.modes.keys() {
            item(*a, ui);
        }
    });

    if let Some(effect) = states.aura.modes.get_mut(&selected) {
        let mut zone_button = |a: AuraZone, ui: &mut Ui| {
            ui.selectable_value(&mut effect.zone, a, format!("{:?}", a));
        };
        let mut speed_button = |a: Speed, ui: &mut Ui| {
            if ui
                .selectable_value(&mut effect.speed, a, format!("{:?}", a))
                .clicked()
            {
                let val = match effect.speed {
                    Speed::Low => 6,
                    Speed::Med => 8,
                    Speed::High => 10,
                };
                freq.store(val, Ordering::SeqCst);
            }
        };
        let mut dir_button = |a: rog_aura::Direction, ui: &mut Ui| {
            ui.selectable_value(&mut effect.direction, a, format!("{:?}", a));
        };

        let mut c1: [u8; 3] = effect.colour1.into();
        let mut c2: [u8; 3] = effect.colour2.into();

        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                let h = 16.0;
                ui.set_row_height(22.0);
                ui.add_enabled_ui(allowed.zone, |ui| {
                    if has_keyzones || has_lightbar || has_logo {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Zone").size(h));
                        });
                    }
                });
                ui.add_enabled_ui(allowed.colour1, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Colour 1").size(h));
                    });
                });
                ui.add_enabled_ui(allowed.colour2, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Colour 2").size(h));
                    });
                });
                ui.add_enabled_ui(allowed.speed, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.set_enabled(allowed.speed);
                        ui.label(RichText::new("Speed").size(h));
                    });
                });
                ui.add_enabled_ui(allowed.direction, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.set_enabled(allowed.direction);
                        ui.label(RichText::new("Direction").size(h));
                    });
                });
                ui.set_enabled(true);
            });
            ui.vertical(|ui| {
                ui.set_row_height(22.0);
                ui.add_enabled_ui(allowed.zone, |ui| {
                    if has_keyzones || has_lightbar || has_logo {
                        ui.horizontal_wrapped(|ui| {
                            zone_button(AuraZone::None, ui);
                            if has_keyzones {
                                zone_button(AuraZone::Key1, ui);
                                zone_button(AuraZone::Key2, ui);
                                zone_button(AuraZone::Key3, ui);
                                zone_button(AuraZone::Key4, ui);
                            }
                            if has_logo {
                                zone_button(AuraZone::Logo, ui);
                            }
                            if has_lightbar {
                                zone_button(AuraZone::BarLeft, ui);
                                zone_button(AuraZone::BarRight, ui);
                            }
                        });
                    }
                });

                ui.add_enabled_ui(allowed.colour1, |ui| {
                    egui::color_picker::color_edit_button_srgb(ui, &mut c1)
                });
                ui.add_enabled_ui(allowed.colour2, |ui| {
                    egui::color_picker::color_edit_button_srgb(ui, &mut c2)
                });

                ui.add_enabled_ui(allowed.speed, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        speed_button(Speed::Low, ui);
                        speed_button(Speed::Med, ui);
                        speed_button(Speed::High, ui);
                    });
                });

                ui.add_enabled_ui(allowed.direction, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        dir_button(rog_aura::Direction::Left, ui);
                        dir_button(rog_aura::Direction::Down, ui);
                        dir_button(rog_aura::Direction::Right, ui);
                        dir_button(rog_aura::Direction::Up, ui);
                    });
                });
            });
        });

        effect.colour1 = Colour::from(&c1);
        effect.colour2 = Colour::from(&c2);
    }

    ui.separator();
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        if ui.add(egui::Button::new("Cancel")).clicked() {
            match AuraState::new(supported, &states.asus_dbus) {
                Ok(a) => states.aura.modes = a.modes,
                Err(e) => states.error = Some(e.to_string()),
            }
        }

        if ui.add(egui::Button::new("Apply")).clicked() {
            changed = true;
        }
    });

    // egui::TopBottomPanel::bottom("error_bar")
    //     .default_height(26.0)
    //     .show(ctx, |ui| {
    //         ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    //             if ui.add(egui::Button::new("Cancel")).clicked() {
    //                 let notif = states.aura.was_notified.clone();
    //                 states.aura.modes = AuraState::new(notif, supported, dbus).modes;
    //             }

    //             if ui.add(egui::Button::new("Apply")).clicked() {
    //                 changed = true;
    //             }
    //         });
    //     });

    if changed {
        states.aura.current_mode = selected;

        states
            .asus_dbus
            .proxies()
            .led()
            .set_led_mode(states.aura.modes.get(&selected).unwrap())
            .map_err(|err| {
                states.error = Some(err.to_string());
            })
            .ok();
    }
}
