use crate::{
    widgets::{anime_power_group, aura_power_group, platform_profile, rog_bios_group},
    RogApp,
};

impl<'a> RogApp<'a> {
    pub fn system_page(&mut self, ctx: &egui::Context) {
        let Self {
            supported,
            states,
            asus_dbus: dbus,
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Experimental application for asusd");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if supported.platform_profile.platform_profile {
                    ui.group(|ui| platform_profile(states, dbus, ui));
                }

                ui.separator();
                ui.group(|ui| rog_bios_group(supported, states, dbus, ui));

                ui.separator();
                ui.group(|ui| aura_power_group(supported, states, dbus, ui));

                if supported.anime_ctrl.0 {
                    ui.separator();
                    ui.group(|ui| anime_power_group(supported, states, dbus, ui));
                }
            });
        });
    }
}
