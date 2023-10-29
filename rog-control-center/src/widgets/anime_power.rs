use egui::{RichText, Ui};
use rog_anime::usb::Brightness;
use rog_platform::supported::SupportedFunctions;

use crate::system_state::SystemState;

pub fn anime_power_group(_supported: &SupportedFunctions, states: &mut SystemState, ui: &mut Ui) {
    ui.heading("AniMe Matrix Settings");
    ui.label("Options are incomplete. Awake + Boot should work");

    ui.horizontal_wrapped(|ui| {
        ui.vertical(|ui| {
            let h = 16.0;
            ui.set_row_height(22.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Display brightness").size(h));
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Display enabled").size(h));
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Animations enabled").size(h));
            });
            // ui.horizontal_wrapped(|ui| {
            //     ui.label(RichText::new("Sleep").size(h));
            // });
        });
        ui.vertical(|ui| {
            ui.set_row_height(22.0);
            ui.horizontal_wrapped(|ui| {
                let slider =
                    egui::Slider::new(&mut states.anime.display_brightness, 0..=3).step_by(1.0);
                if ui.add(slider).drag_released() {
                    states
                        .asus_dbus
                        .proxies()
                        .anime()
                        .set_brightness(Brightness::from(states.anime.display_brightness))
                        .map_err(|err| {
                            states.error = Some(err.to_string());
                        })
                        .ok();
                }
            });
            ui.horizontal_wrapped(|ui| {
                if ui
                    .checkbox(&mut states.anime.display_enabled, "Enable")
                    .changed()
                {
                    states
                        .asus_dbus
                        .proxies()
                        .anime()
                        .set_enable_display(states.anime.display_enabled)
                        .map_err(|err| {
                            states.error = Some(err.to_string());
                        })
                        .ok();
                }
            });
            ui.horizontal_wrapped(|ui| {
                if ui
                    .checkbox(&mut states.anime.builtin_anims_enabled, "Enable")
                    .changed()
                {
                    states
                        .asus_dbus
                        .proxies()
                        .anime()
                        .set_builtins_enabled(states.anime.builtin_anims_enabled)
                        .map_err(|err| {
                            states.error = Some(err.to_string());
                        })
                        .ok();
                }
            });
            // ui.horizontal_wrapped(|ui| {
            //     if ui.checkbox(&mut states.anime.sleep, "Enable").changed() {
            //         changed = true;
            //     }
            // });
        });
    });
}
