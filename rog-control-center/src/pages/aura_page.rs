use egui::Color32;

use crate::{
    widgets::{aura_modes_group, keyboard},
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

        let c = states
            .aura
            .modes
            .get(&states.aura.current_mode)
            .unwrap()
            .colour1;
        let colour = Color32::from_rgb(c.0, c.1, c.2);
        // TODO: animation of colour changes/periods/blending
        egui::CentralPanel::default().show(ctx, |ui| {
            aura_modes_group(supported, states, dbus, ui);

            keyboard(ui, &states.keyboard_layout, colour);
        });
    }
}
