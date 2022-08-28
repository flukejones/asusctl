use crate::{Page, RogApp};

impl<'a> RogApp<'a> {
    pub fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .default_width(60.0) // TODO: set size to match icon buttons when done
            .show(ctx, |ui| {
                let Self { page, .. } = self;

                ui.heading("Functions");

                ui.separator();
                if ui
                    .selectable_value(page, Page::System, "System Settings")
                    .clicked()
                {
                    *page = Page::System;
                }

                if self.supported.platform_profile.fan_curves {
                    ui.separator();
                    if ui
                        .selectable_value(page, Page::FanCurves, "Fan Curves")
                        .clicked()
                    {
                        *page = Page::FanCurves;
                    }
                }

                if !self.supported.keyboard_led.stock_led_modes.is_empty() {
                    ui.separator();
                    if ui
                        .selectable_value(page, Page::AuraEffects, "Keyboard Aura")
                        .clicked()
                    {
                        *page = Page::AuraEffects;
                    }
                }

                // TODO: Anime page is not complete
                // if self.supported.anime_ctrl.0 {
                //     ui.separator();
                //     if ui
                //         .selectable_value(page, Page::AnimeMatrix, "AniMe Matrix")
                //         .clicked()
                //     {
                //         *page = Page::AnimeMatrix;
                //     }
                // }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("Source code ");
                        ui.hyperlink_to("rog-gui.", "https://gitlab.com/asus-linux/asusctl/-/tree/main/rog-control-center");
                    });
                });
            });
    }
}
