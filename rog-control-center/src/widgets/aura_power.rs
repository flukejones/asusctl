use egui::{RichText, Ui};
use rog_aura::power::{AuraPower, KbAuraPowerState};
use rog_aura::usb::{AuraDevRog1, AuraDevTuf, AuraDevice, AuraPowerDev};

use crate::system_state::SystemState;

pub fn aura_power_group(states: &mut SystemState, ui: &mut Ui) {
    ui.heading("Keyboard LED power settings");

    if states.aura.dev_type.is_old_style() || states.aura.dev_type.is_tuf_style() {
        aura_power1(states, ui);
    } else if states.aura.dev_type.is_new_style() {
        aura_power2(states, ui);
    }
}

fn aura_power1(states: &mut SystemState, ui: &mut Ui) {
    let enabled_states = &mut states.aura.enabled;
    let mut boot = enabled_states.old_rog.contains(&AuraDevRog1::Boot);
    let mut sleep = enabled_states.old_rog.contains(&AuraDevRog1::Sleep);
    let mut keyboard = enabled_states.old_rog.contains(&AuraDevRog1::Keyboard);
    let mut lightbar = enabled_states.old_rog.contains(&AuraDevRog1::Lightbar);
    if states.aura.dev_type == AuraDevice::Tuf {
        boot = enabled_states.tuf.contains(&AuraDevTuf::Boot);
        sleep = enabled_states.tuf.contains(&AuraDevTuf::Sleep);
        keyboard = enabled_states.tuf.contains(&AuraDevTuf::Awake);
    }
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
            // if supported.keyboard_led.brightness_set {
            //     ui.horizontal_wrapped(|ui| {
            //         ui.label(RichText::new("Brightness").size(h));
            //     });
            // }
        });
        ui.vertical(|ui| {
            ui.set_row_height(22.0);
            ui.horizontal_wrapped(|ui| {
                if ui.checkbox(&mut boot, "Enable").changed() {
                    changed = true;
                }
            });
            ui.horizontal_wrapped(|ui| {
                if ui.toggle_value(&mut keyboard, "Keyboard").changed() {
                    changed = true;
                }
                if !states.aura.supported_basic_zones.is_empty()
                    && ui.toggle_value(&mut lightbar, "Lightbar").changed()
                {
                    changed = true;
                }
            });
            ui.horizontal_wrapped(|ui| {
                if ui.checkbox(&mut sleep, "Enable").changed() {
                    changed = true;
                }
            });

            // We currently don't have a watch for system changes here
            // if supported.keyboard_led.brightness_set {
            //     if ui
            //         .add(egui::Slider::new(
            //             &mut states.aura.bright,
            //             0..=3,
            //         ))
            //         .changed()
            //     {
            //         let bright = LedBrightness::from(states.aura.bright as
            // u32);         dbus.proxies()
            //             .led()
            //             .set_brightness(bright)
            //             .map_err(|err| {
            //                 states.error = Some(err.to_string());
            //             })
            //             .ok();
            //     }
            // }
        });
    });

    if changed {
        if states.aura.dev_type == AuraDevice::Tuf {
            let mut enabled = Vec::new();
            let mut disabled = Vec::new();

            let mut modify_tuf = |b: bool, a: AuraDevTuf| {
                if b {
                    enabled.push(a);
                    if !enabled_states.tuf.contains(&a) {
                        enabled_states.tuf.push(a);
                    }
                } else {
                    disabled.push(a);
                    // This would be so much better as a hashset
                    if enabled_states.tuf.contains(&a) {
                        let mut idx = 0;
                        for (i, n) in enabled_states.tuf.iter().enumerate() {
                            if *n == a {
                                idx = i;
                                break;
                            }
                        }
                        enabled_states.tuf.remove(idx);
                    }
                }
            };
            modify_tuf(boot, AuraDevTuf::Boot);
            modify_tuf(sleep, AuraDevTuf::Sleep);
            modify_tuf(keyboard, AuraDevTuf::Awake);

            let mut send = |enable: bool, data: Vec<AuraDevTuf>| {
                let options = AuraPowerDev {
                    tuf: data,
                    ..Default::default()
                };
                // build data to send
                states
                    .asus_dbus
                    .proxies()
                    .aura()
                    .set_led_power((options, enable))
                    .map_err(|err| {
                        dbg!(&err);
                        states.error = Some(err.to_string());
                    })
                    .ok();
            };
            send(true, enabled);
            send(false, disabled);
        } else {
            let mut enabled = Vec::new();
            let mut disabled = Vec::new();

            let mut modify_x1866 = |b: bool, a: AuraDevRog1| {
                if b {
                    enabled.push(a);
                    if !enabled_states.old_rog.contains(&a) {
                        enabled_states.old_rog.push(a);
                    }
                } else {
                    disabled.push(a);
                    // This would be so much better as a hashset
                    if enabled_states.old_rog.contains(&a) {
                        let mut idx = 0;
                        for (i, n) in enabled_states.old_rog.iter().enumerate() {
                            if *n == a {
                                idx = i;
                                break;
                            }
                        }
                        enabled_states.old_rog.remove(idx);
                    }
                }
            };
            modify_x1866(boot, AuraDevRog1::Boot);
            modify_x1866(sleep, AuraDevRog1::Sleep);
            modify_x1866(keyboard, AuraDevRog1::Keyboard);
            if !states.aura.supported_basic_zones.is_empty() {
                modify_x1866(lightbar, AuraDevRog1::Lightbar);
            }

            let mut send = |enable: bool, data: Vec<AuraDevRog1>| {
                let options = AuraPowerDev {
                    old_rog: data,
                    ..Default::default()
                };
                // build data to send
                states
                    .asus_dbus
                    .proxies()
                    .aura()
                    .set_led_power((options, enable))
                    .map_err(|err| {
                        dbg!(&err);
                        states.error = Some(err.to_string());
                    })
                    .ok();
            };
            send(true, enabled);
            send(false, disabled);
        }
    }
}

fn aura_power2(states: &mut SystemState, ui: &mut Ui) {
    let AuraPower {
        keyboard,
        logo,
        lightbar,
        lid,
        rear_glow,
    } = &mut states.aura.enabled.rog;

    const LABELS: [&str; 4] = ["Boot", "Awake", "Sleep", "Shutdown"];

    let mut changed = false;
    let mut item = |power: &mut KbAuraPowerState, ui: &mut Ui| {
        ui.vertical(|ui| {
            if states.aura.supported_power_zones.contains(&power.zone) {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(format!("{:?}", power.zone)).size(14.0));
                });
                if ui.checkbox(&mut power.boot, LABELS[0]).changed() {
                    changed = true;
                }
                if ui.checkbox(&mut power.awake, LABELS[1]).changed() {
                    changed = true;
                }
                if ui.checkbox(&mut power.sleep, LABELS[2]).changed() {
                    changed = true;
                }
                if ui.checkbox(&mut power.shutdown, LABELS[3]).changed() {
                    changed = true;
                }
            }
        });
    };

    ui.horizontal_wrapped(|ui| {
        item(lid, ui);
        item(logo, ui);
        item(keyboard, ui);
        item(lightbar, ui);
        item(rear_glow, ui);
    });

    if changed {
        let mut send = |enable: bool, data: AuraPower| {
            let options = AuraPowerDev {
                rog: data,
                ..Default::default()
            };
            // build data to send
            states
                .asus_dbus
                .proxies()
                .aura()
                .set_led_power((options, enable))
                .map_err(|err| {
                    dbg!(&err);
                    states.error = Some(err.to_string());
                })
                .ok();
        };
        send(
            true,
            AuraPower {
                keyboard: *keyboard,
                logo: *logo,
                lightbar: *lightbar,
                lid: *lid,
                rear_glow: *rear_glow,
            },
        );
    }
}
