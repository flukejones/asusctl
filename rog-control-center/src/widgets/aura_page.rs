use egui::{RichText, Ui};
use rog_aura::{
    usb::{AuraDev1866, AuraDev19b6, AuraDevice, AuraPowerDev},
    AuraModeNum, AuraZone, Colour, Speed,
};
use rog_dbus::RogDbusClientBlocking;
use rog_supported::SupportedFunctions;

use crate::{
    page_states::{AuraState, PageDataStates},
    RogApp,
};

impl<'a> RogApp<'a> {
    pub fn aura_page(&mut self, ctx: &egui::Context) {
        let Self {
            supported,
            states,
            asus_dbus: dbus,
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            Self::aura_power(supported, states, dbus, ui);
            ui.separator();
            Self::aura_modes(supported, states, dbus, ui);
        });
    }

    fn aura_power(
        supported: &SupportedFunctions,
        states: &mut PageDataStates,
        dbus: &mut RogDbusClientBlocking,
        ui: &mut Ui,
    ) {
        match supported.keyboard_led.prod_id {
            AuraDevice::X1854 | AuraDevice::X1869 | AuraDevice::X1866 => {
                Self::aura_power1(supported, states, dbus, ui)
            }
            AuraDevice::X19B6 => Self::aura_power2(supported, states, dbus, ui),
            AuraDevice::Unknown => {}
        }
    }

    fn aura_power1(
        supported: &SupportedFunctions,
        states: &mut PageDataStates,
        dbus: &mut RogDbusClientBlocking,
        ui: &mut Ui,
    ) {
        let enabled_states = &mut states.aura.enabled;

        ui.heading("Aura go brrrrr! (incomplete)");
        ui.separator();

        let boot = &mut enabled_states.x1866.contains(&AuraDev1866::Boot);
        let sleep = &mut enabled_states.x1866.contains(&AuraDev1866::Sleep);
        let keyboard = &mut enabled_states.x1866.contains(&AuraDev1866::Keyboard);
        let lightbar = &mut enabled_states.x1866.contains(&AuraDev1866::Lightbar);
        let mut changed = false;

        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                let h = 16.0;
                ui.set_row_height(22.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Boot").size(h));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Awake").size(h));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Sleep").size(h));
                });
            });
            ui.vertical(|ui| {
                ui.set_row_height(22.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.checkbox(boot, "Enable").changed() {
                        changed = true;
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.toggle_value(keyboard, "Keyboard").changed() {
                        changed = true;
                    }
                    if !supported.keyboard_led.multizone_led_mode.is_empty() {
                        if ui.toggle_value(lightbar, "Lightbar").changed() {
                            changed = true;
                        }
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.checkbox(sleep, "Enable").changed() {
                        changed = true;
                    }
                });
            });
        });

        if changed {
            let mut enabled = Vec::new();
            let mut disabled = Vec::new();

            let mut modify = |b: bool, a: AuraDev1866| {
                if b {
                    enabled.push(a);
                    if !enabled_states.x1866.contains(&a) {
                        enabled_states.x1866.push(a);
                    }
                } else {
                    disabled.push(a);
                    // This would be so much better as a hashset
                    if enabled_states.x1866.contains(&a) {
                        let mut idx = 0;
                        for (i, n) in enabled_states.x1866.iter().enumerate() {
                            if *n == a {
                                idx = i;
                                break;
                            }
                        }
                        enabled_states.x1866.remove(idx);
                    }
                }
            };
            modify(*boot, AuraDev1866::Boot);
            modify(*sleep, AuraDev1866::Sleep);
            modify(*keyboard, AuraDev1866::Keyboard);
            if !supported.keyboard_led.multizone_led_mode.is_empty() {
                modify(*lightbar, AuraDev1866::Lightbar);
            }

            let mut send = |enable: bool, data: Vec<AuraDev1866>| {
                let options = AuraPowerDev {
                    x1866: data,
                    x19b6: vec![],
                };
                // build data to send
                dbus.proxies()
                    .led()
                    .set_leds_power(options, enable)
                    .map_err(|err| {
                        states.error = Some(err.to_string());
                    })
                    .ok();
            };
            send(true, enabled);
            send(false, disabled);
        }
    }

    fn aura_power2(
        supported: &SupportedFunctions,
        states: &mut PageDataStates,
        dbus: &mut RogDbusClientBlocking,
        ui: &mut Ui,
    ) {
        let enabled_states = &mut states.aura.enabled;

        ui.heading("Lights go brrrrr! (incomplete)");
        ui.separator();

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

        let boot_bar = &mut enabled_states.x19b6.contains(&AuraDev19b6::BootBar);
        let boot_logo = &mut enabled_states.x19b6.contains(&AuraDev19b6::BootLogo);
        let boot_keyb = &mut enabled_states.x19b6.contains(&AuraDev19b6::BootKeyb);

        let awake_bar = &mut enabled_states.x19b6.contains(&AuraDev19b6::AwakeBar);
        let awake_logo = &mut enabled_states.x19b6.contains(&AuraDev19b6::AwakeLogo);
        let awake_keyb = &mut enabled_states.x19b6.contains(&AuraDev19b6::AwakeKeyb);

        let sleep_bar = &mut enabled_states.x19b6.contains(&AuraDev19b6::SleepBar);
        let sleep_logo = &mut enabled_states.x19b6.contains(&AuraDev19b6::SleepLogo);
        let sleep_keyb = &mut enabled_states.x19b6.contains(&AuraDev19b6::SleepKeyb);

        let mut changed = false;

        let mut item = |keyboard: &mut bool, logo: &mut bool, lightbar: &mut bool, ui: &mut Ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.checkbox(keyboard, "Keyboard").changed() {
                    changed = true;
                }
                if has_logo && ui.checkbox(logo, "Logo").changed() {
                    changed = true;
                }
                if has_lightbar && ui.checkbox(lightbar, "Lightbar").changed() {
                    changed = true;
                }
            });
        };

        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                let h = 16.0;
                ui.set_row_height(22.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Boot").size(h));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Awake").size(h));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Sleep").size(h));
                });
            });
            ui.vertical(|ui| {
                ui.set_row_height(22.0);
                item(boot_keyb, boot_logo, boot_bar, ui);
                item(awake_keyb, awake_logo, awake_bar, ui);
                item(sleep_keyb, sleep_logo, sleep_bar, ui);
            });
        });

        if changed {
            let mut enabled = Vec::new();
            let mut disabled = Vec::new();

            let mut modify = |b: bool, a: AuraDev19b6| {
                if b {
                    enabled.push(a);
                    if !enabled_states.x19b6.contains(&a) {
                        enabled_states.x19b6.push(a);
                    }
                } else {
                    disabled.push(a);
                    // This would be so much better as a hashset
                    if enabled_states.x19b6.contains(&a) {
                        let mut idx = 0;
                        for (i, n) in enabled_states.x19b6.iter().enumerate() {
                            if *n == a {
                                idx = i;
                                break;
                            }
                        }
                        enabled_states.x1866.remove(idx);
                    }
                }
            };
            modify(*boot_keyb, AuraDev19b6::BootKeyb);
            modify(*sleep_keyb, AuraDev19b6::SleepKeyb);
            modify(*awake_keyb, AuraDev19b6::AwakeKeyb);
            if supported
                .keyboard_led
                .multizone_led_mode
                .contains(&AuraZone::Logo)
            {
                modify(*boot_logo, AuraDev19b6::BootLogo);
                modify(*sleep_logo, AuraDev19b6::SleepLogo);
                modify(*awake_logo, AuraDev19b6::AwakeLogo);
            }
            if supported
                .keyboard_led
                .multizone_led_mode
                .contains(&AuraZone::BarLeft)
            {
                modify(*boot_bar, AuraDev19b6::BootBar);
                modify(*sleep_bar, AuraDev19b6::SleepBar);
                modify(*awake_bar, AuraDev19b6::AwakeBar);
            }

            let mut send = |enable: bool, data: Vec<AuraDev19b6>| {
                let options = AuraPowerDev {
                    x1866: vec![],
                    x19b6: data,
                };
                // build data to send
                dbus.proxies()
                    .led()
                    .set_leds_power(options, enable)
                    .map_err(|err| {
                        states.error = Some(err.to_string());
                    })
                    .ok();
            };
            send(true, enabled);
            send(false, disabled);
        }
    }

    fn aura_modes(
        supported: &SupportedFunctions,
        states: &mut PageDataStates,
        dbus: &mut RogDbusClientBlocking,
        ui: &mut Ui,
    ) {
        let mut changed = false;
        let mut selected = states.aura.current_mode;

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

        // TODO: Need some sort of mapping to enable options only if
        //  they actually work.
        if let Some(effect) = states.aura.modes.get_mut(&selected) {
            let mut zone_button = |a: AuraZone, ui: &mut Ui| {
                ui.selectable_value(&mut effect.zone, a, format!("{:?}", a));
            };
            let mut speed_button = |a: Speed, ui: &mut Ui| {
                ui.selectable_value(&mut effect.speed, a, format!("{:?}", a));
            };
            let mut dir_button = |a: rog_aura::Direction, ui: &mut Ui| {
                ui.selectable_value(&mut effect.direction, a, format!("{:?}", a));
            };

            let mut c1: [f32; 3] = effect.colour1.into();
            let mut c2: [f32; 3] = effect.colour2.into();

            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    let h = 16.0;
                    ui.set_row_height(22.0);
                    if has_keyzones || has_lightbar || has_logo {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Zone").size(h));
                        });
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Colour 1").size(h));
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Colour 2").size(h));
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Speed").size(h));
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Direction").size(h));
                    });
                });
                ui.vertical(|ui| {
                    ui.set_row_height(22.0);
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

                    egui::color_picker::color_edit_button_rgb(ui, &mut c1);
                    egui::color_picker::color_edit_button_rgb(ui, &mut c2);

                    ui.horizontal_wrapped(|ui| {
                        speed_button(Speed::Low, ui);
                        speed_button(Speed::Med, ui);
                        speed_button(Speed::High, ui);
                    });

                    ui.horizontal_wrapped(|ui| {
                        dir_button(rog_aura::Direction::Left, ui);
                        dir_button(rog_aura::Direction::Down, ui);
                        dir_button(rog_aura::Direction::Right, ui);
                        dir_button(rog_aura::Direction::Up, ui);
                    });
                });
            });

            effect.colour1 = Colour::from(&c1);
            effect.colour2 = Colour::from(&c2);
        }

        ui.separator();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui.add(egui::Button::new("Cancel")).clicked() {
                let notif = states.aura.was_notified.clone();
                states.aura.modes = AuraState::new(notif, supported, dbus).modes;
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

            dbus.proxies()
                .led()
                .set_led_mode(states.aura.modes.get(&selected).unwrap())
                .map_err(|err| {
                    states.error = Some(err.to_string());
                })
                .ok();
        }
    }
}
