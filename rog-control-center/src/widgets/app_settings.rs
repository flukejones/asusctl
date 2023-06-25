use egui::Ui;

use crate::config::Config;
use crate::system_state::SystemState;

pub fn app_settings(config: &mut Config, states: &mut SystemState, ui: &mut Ui) {
    ui.heading("ROG GUI Settings");
    // ui.label("Options are incomplete. Awake + Boot should work");

    let mut enabled_notifications = if let Ok(lock) = states.enabled_notifications.lock() {
        lock.clone()
    } else {
        Default::default()
    };

    ui.label("Application settings");
    let app_changed = ui
        .checkbox(&mut config.run_in_background, "Run in Background")
        .clicked()
        || ui
            .checkbox(&mut config.startup_in_background, "Startup Hidden")
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.all_enabled,
                "Enable Notifications",
            )
            .clicked();

    ui.label("Notification settings");
    let notif_changed = ui
        .checkbox(
            &mut enabled_notifications.receive_notify_gfx_status,
            "Enable dGPU status notification",
        )
        .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_notify_led,
                "Enable LED mode change notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_notify_dgpu_disable,
                "Enable dGPU disablement notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_notify_egpu_enable,
                "Enable eGPU enablement notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_notify_mains_online,
                "Enable mains (AC) power notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_notify_charge_control_end_threshold,
                "Enable charge threshold notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_notify_profile,
                "Enable profile change notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_notify_panel_od,
                "Enable panel overdrive notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_notify_mini_led_mode,
                "Enable MiniLED mode notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_notify_post_boot_sound,
                "Enable BIOS post sound notification",
            )
            .clicked();

    if app_changed || notif_changed {
        if let Ok(mut lock) = states.enabled_notifications.lock() {
            // Replace inner content before save
            *lock = enabled_notifications;

            config
                .save(&lock)
                .map_err(|err| {
                    states.error = Some(err.to_string());
                })
                .ok();
        }
    }
}
