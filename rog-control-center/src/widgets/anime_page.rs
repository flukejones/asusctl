use egui::RichText;

use crate::RogApp;

impl<'a> RogApp<'a> {
    pub fn anime_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AniMe Matrix Settings");
            ui.label("Options are incomplete. Awake + Boot should work");

            let Self {
                states,
                asus_dbus: dbus,
                ..
            } = self;

            let mut changed = false;

            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    let h = 16.0;
                    ui.set_row_height(22.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Brightness").size(h));
                    });
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
                        if ui
                            .add(egui::Slider::new(&mut states.anime.bright, 0..=254))
                            .changed()
                        {
                            changed = true;
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        if ui.checkbox(&mut states.anime.boot, "Enable").changed() {
                            dbus.proxies()
                                .anime()
                                .set_boot_on_off(states.anime.boot)
                                .map_err(|err| {
                                    states.error = Some(err.to_string());
                                })
                                .ok();
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        if ui.checkbox(&mut states.anime.awake, "Enable").changed() {
                            dbus.proxies()
                                .anime()
                                .set_on_off(states.anime.awake)
                                .map_err(|err| {
                                    states.error = Some(err.to_string());
                                })
                                .ok();
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        if ui.checkbox(&mut states.anime.sleep, "Enable").changed() {
                            changed = true;
                        }
                    });
                });
            });
        });
    }
}
