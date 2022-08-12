use crate::{page_states::PageDataStates, RogDbusClientBlocking};
use egui::Ui;
use rog_profiles::Profile;
use rog_supported::SupportedFunctions;

pub fn platform_profile(states: &mut PageDataStates, dbus: &RogDbusClientBlocking, ui: &mut Ui) {
    ui.heading("Platform profile");

    let mut changed = false;
    let mut item = |p: Profile, ui: &mut Ui| {
        if ui
            .selectable_value(&mut states.profiles.current, p, format!("{p:?}"))
            .clicked()
        {
            changed = true;
        }
    };

    ui.horizontal_wrapped(|ui| {
        for a in states.profiles.list.iter() {
            item(*a, ui);
        }
    });

    if changed {
        dbus.proxies()
            .profile()
            .set_active_profile(states.profiles.current)
            .map_err(|err| {
                states.error = Some(err.to_string());
            })
            .ok();
    };
}

pub fn rog_bios_group(
    supported: &SupportedFunctions,
    states: &mut PageDataStates,
    dbus: &mut RogDbusClientBlocking,
    ui: &mut Ui,
) {
    ui.heading("Bios options");

    let slider = egui::Slider::new(&mut states.charge_limit, 20..=100)
        .text("Charging limit")
        .step_by(1.0);
    if ui.add(slider).drag_released() {
        dbus.proxies()
            .charge()
            .set_limit(states.charge_limit as u8)
            .map_err(|err| {
                states.error = Some(err.to_string());
            })
            .ok();
    }

    if supported.rog_bios_ctrl.post_sound {
        if ui
            .add(egui::Checkbox::new(
                &mut states.bios.post_sound,
                "POST sound",
            ))
            .changed()
        {
            dbus.proxies()
                .rog_bios()
                .set_post_boot_sound(states.bios.post_sound)
                .map_err(|err| {
                    states.error = Some(err.to_string());
                })
                .ok();
        }
    }

    if supported.rog_bios_ctrl.post_sound {
        if ui
            .add(egui::Checkbox::new(
                &mut states.bios.panel_overdrive,
                "Panel overdrive",
            ))
            .changed()
        {
            dbus.proxies()
                .rog_bios()
                .set_panel_overdrive(states.bios.panel_overdrive)
                .map_err(|err| {
                    states.error = Some(err.to_string());
                })
                .ok();
        }
    }

    if supported.rog_bios_ctrl.dgpu_only {
        if ui
            .add(egui::Checkbox::new(
                &mut states.bios.dedicated_gfx,
                "G-Sync Dedicated GPU mode",
            ))
            .changed()
        {
            dbus.proxies()
                .rog_bios()
                .set_dedicated_graphic_mode(states.bios.dedicated_gfx)
                .map_err(|err| {
                    states.error = Some(err.to_string());
                })
                .ok();
        }
    }
}
