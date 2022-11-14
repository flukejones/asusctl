use egui::{RichText, Ui};
use rog_aura::{
    usb::{AuraDev1866, AuraDev19b6, AuraDevTuf, AuraDevice, AuraPowerDev},
    AuraZone,
};
use rog_platform::supported::SupportedFunctions;

use crate::page_states::PageDataStates;

pub fn aura_power_group(supported: &SupportedFunctions, states: &mut PageDataStates, ui: &mut Ui) {
    ui.heading("LED settings");

    match supported.keyboard_led.prod_id {
        AuraDevice::X1854 | AuraDevice::X1869 | AuraDevice::X1866 => {
            aura_power1(supported, states, ui)
        }
        AuraDevice::X19B6 => aura_power2(supported, states, ui),
        AuraDevice::Tuf => aura_power1(supported, states, ui),
        AuraDevice::Unknown => {}
    }
}

fn aura_power1(supported: &SupportedFunctions, states: &mut PageDataStates, ui: &mut Ui) {
    let enabled_states = &mut states.aura.enabled;
    let mut boot = enabled_states.x1866.contains(&AuraDev1866::Boot);
    let mut sleep = enabled_states.x1866.contains(&AuraDev1866::Sleep);
    let mut keyboard = enabled_states.x1866.contains(&AuraDev1866::Keyboard);
    let mut lightbar = enabled_states.x1866.contains(&AuraDev1866::Lightbar);
    if supported.keyboard_led.prod_id == AuraDevice::Tuf {
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
                if !supported.keyboard_led.multizone_led_mode.is_empty()
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
            //         let bright = LedBrightness::from(states.aura.bright as u32);
            //         dbus.proxies()
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
        if supported.keyboard_led.prod_id == AuraDevice::Tuf {
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
                    x1866: vec![],
                    x19b6: vec![],
                };
                // build data to send
                states
                    .asus_dbus
                    .proxies()
                    .led()
                    .set_leds_power(options, enable)
                    .map_err(|err| {
                        states.error = Some(err.to_string());
                    })
                    .ok();
            };
            send(true, enabled);
            send(false, disabled);
        } else {
            let mut enabled = Vec::new();
            let mut disabled = Vec::new();

            let mut modify_x1866 = |b: bool, a: AuraDev1866| {
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
            modify_x1866(boot, AuraDev1866::Boot);
            modify_x1866(sleep, AuraDev1866::Sleep);
            modify_x1866(keyboard, AuraDev1866::Keyboard);
            if !supported.keyboard_led.multizone_led_mode.is_empty() {
                modify_x1866(lightbar, AuraDev1866::Lightbar);
            }

            let mut send = |enable: bool, data: Vec<AuraDev1866>| {
                let options = AuraPowerDev {
                    tuf: vec![],
                    x1866: data,
                    x19b6: vec![],
                };
                // build data to send
                states
                    .asus_dbus
                    .proxies()
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
}

fn aura_power2(supported: &SupportedFunctions, states: &mut PageDataStates, ui: &mut Ui) {
    let enabled_states = &mut states.aura.enabled;
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

    let boot_bar = &mut enabled_states.x19b6.contains(&AuraDev19b6::AwakeBar);
    let boot_logo = &mut enabled_states.x19b6.contains(&AuraDev19b6::BootLogo);
    let boot_keyb = &mut enabled_states.x19b6.contains(&AuraDev19b6::BootKeyb);

    let awake_bar = &mut enabled_states.x19b6.contains(&AuraDev19b6::BootBar);
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
                    enabled_states.x19b6.remove(idx);
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
            modify(*boot_bar, AuraDev19b6::AwakeBar);
            modify(*sleep_bar, AuraDev19b6::SleepBar);
            modify(*awake_bar, AuraDev19b6::BootBar);
        }

        let mut send = |enable: bool, data: Vec<AuraDev19b6>| {
            let options = AuraPowerDev {
                tuf: vec![],
                x1866: vec![],
                x19b6: data,
            };
            // build data to send
            states
                .asus_dbus
                .proxies()
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
