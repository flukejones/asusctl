use egui::Ui;

use crate::config::Config;
use crate::system_state::SystemState;

pub fn app_settings(config: &mut Config, states: &mut SystemState, ui: &mut Ui) {
    ui.heading("App Settings");

    let mut enabled_notifications = if let Ok(lock) = states.enabled_notifications.lock() {
        lock.clone()
    } else {
        Default::default()
    };

    ui.label("Application settings");
    let app_changed = ui
        .checkbox(
            &mut config.enable_tray_icon,
            "Enable Tray Icon (restart required)",
        )
        .clicked()
        || ui
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

    // if ui.button("Quit").clicked() {
    //     states.run_in_bg = false;
    // }

    ui.label("Notification settings");
    let notif_changed = ui
        .checkbox(
            &mut enabled_notifications.receive_notify_gfx_status,
            "Enable dGPU status notification",
        )
        .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_led_mode_data_changed,
                "Enable LED mode change notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_dgpu_disable_changed,
                "Enable dGPU disablement notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_egpu_enable_changed,
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
                &mut enabled_notifications.receive_charge_control_end_threshold_changed,
                "Enable charge threshold notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_throttle_thermal_policy_changed,
                "Enable profile change notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_panel_od_changed,
                "Enable panel overdrive notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_mini_led_mode_changed,
                "Enable MiniLED mode notification",
            )
            .clicked()
        || ui
            .checkbox(
                &mut enabled_notifications.receive_post_animation_sound_changed,
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

            states.tray_enabled = config.enable_tray_icon;
            states.run_in_bg = config.run_in_background;
        }
    }
}
