use std::sync::atomic::Ordering;

use egui::Ui;

use crate::{config::Config, page_states::PageDataStates};

pub fn app_settings(config: &mut Config, states: &mut PageDataStates, ui: &mut Ui) {
    ui.heading("ROG GUI Settings");
    // ui.label("Options are incomplete. Awake + Boot should work");

    if ui
        .checkbox(&mut config.run_in_background, "Run in Background")
        .clicked()
        || ui
            .checkbox(&mut config.startup_in_background, "Startup Hidden")
            .clicked()
        || ui
            .checkbox(&mut config.enable_notifications, "Enable Notifications")
            .clicked()
    {
        states
            .notifs_enabled
            .store(config.enable_notifications, Ordering::SeqCst);
        config
            .save()
            .map_err(|err| {
                states.error = Some(err.to_string());
            })
            .ok();
    }
}
