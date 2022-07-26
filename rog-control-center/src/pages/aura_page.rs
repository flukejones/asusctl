use crate::{widgets::aura_modes_group, RogApp};

impl<'a> RogApp<'a> {
    pub fn aura_page(&mut self, ctx: &egui::Context) {
        let Self {
            supported,
            states,
            asus_dbus: dbus,
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            aura_modes_group(supported, states, dbus, ui);
        });
    }
}
