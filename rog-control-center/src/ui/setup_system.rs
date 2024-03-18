use std::sync::{Arc, Mutex};

use rog_dbus::zbus_platform::{PlatformProxy, PlatformProxyBlocking};
use rog_platform::platform::Properties;
use slint::ComponentHandle;

use super::show_toast;
use crate::config::Config;
use crate::{
    set_ui_callbacks, set_ui_props_async, AvailableSystemProperties, MainWindow, SystemPageData,
};

pub fn setup_system_page(ui: &MainWindow, _config: Arc<Mutex<Config>>) {
    let conn = zbus::blocking::Connection::system().unwrap();
    let platform = PlatformProxyBlocking::new(&conn).unwrap();

    let sys_props = platform.supported_properties().unwrap();
    log::debug!("Available system properties: {sys_props:?}");
    let props = AvailableSystemProperties {
        ac_command: true,
        bat_command: true,
        charge_control_end_threshold: sys_props.contains(&Properties::ChargeControlEndThreshold),
        disable_nvidia_powerd_on_battery: true,
        mini_led_mode: sys_props.contains(&Properties::MiniLedMode),
        nv_dynamic_boost: sys_props.contains(&Properties::NvDynamicBoost),
        nv_temp_target: sys_props.contains(&Properties::NvTempTarget),
        panel_od: sys_props.contains(&Properties::PanelOd),
        boot_sound: sys_props.contains(&Properties::PostAnimationSound),
        ppt_apu_sppt: sys_props.contains(&Properties::PptApuSppt),
        ppt_fppt: sys_props.contains(&Properties::PptFppt),
        ppt_pl1_spl: sys_props.contains(&Properties::PptPl1Spl),
        ppt_pl2_sppt: sys_props.contains(&Properties::PptPl2Sppt),
        ppt_platform_sppt: sys_props.contains(&Properties::PptPlatformSppt),
        throttle_thermal_policy: sys_props.contains(&Properties::ThrottlePolicy),
    };

    ui.global::<SystemPageData>().set_available(props);
}

pub fn setup_system_page_callbacks(ui: &MainWindow, _states: Arc<Mutex<Config>>) {
    // This tokio spawn exists only to prevent blocking the UI, and to enable use of
    // async zbus interfaces
    let handle = ui.as_weak();

    tokio::spawn(async move {
        // Create the connections/proxies here to prevent future delays in process
        let conn = zbus::Connection::system().await.unwrap();
        let platform = PlatformProxy::new(&conn).await.unwrap();

        set_ui_props_async!(
            handle,
            platform,
            SystemPageData,
            charge_control_end_threshold
        );
        set_ui_props_async!(handle, platform, SystemPageData, throttle_thermal_policy);

        set_ui_props_async!(handle, platform, SystemPageData, throttle_policy_linked_epp);
        set_ui_props_async!(handle, platform, SystemPageData, throttle_balanced_epp);
        set_ui_props_async!(handle, platform, SystemPageData, throttle_performance_epp);
        set_ui_props_async!(handle, platform, SystemPageData, throttle_quiet_epp);
        set_ui_props_async!(handle, platform, SystemPageData, throttle_policy_on_battery);
        set_ui_props_async!(handle, platform, SystemPageData, throttle_policy_on_ac);

        set_ui_props_async!(handle, platform, SystemPageData, panel_od);
        set_ui_props_async!(handle, platform, SystemPageData, boot_sound);
        set_ui_props_async!(handle, platform, SystemPageData, mini_led_mode);
        set_ui_props_async!(handle, platform, SystemPageData, ppt_pl1_spl);
        set_ui_props_async!(handle, platform, SystemPageData, ppt_pl2_sppt);
        set_ui_props_async!(handle, platform, SystemPageData, ppt_fppt);
        set_ui_props_async!(handle, platform, SystemPageData, ppt_apu_sppt);
        set_ui_props_async!(handle, platform, SystemPageData, ppt_platform_sppt);
        set_ui_props_async!(handle, platform, SystemPageData, nv_dynamic_boost);
        set_ui_props_async!(handle, platform, SystemPageData, nv_temp_target);

        let sys_props = platform.supported_properties().await.unwrap();
        log::debug!("Available system properties: {sys_props:?}");
        let props = AvailableSystemProperties {
            ac_command: true,
            bat_command: true,
            charge_control_end_threshold: sys_props
                .contains(&Properties::ChargeControlEndThreshold),
            disable_nvidia_powerd_on_battery: true,
            mini_led_mode: sys_props.contains(&Properties::MiniLedMode),
            nv_dynamic_boost: sys_props.contains(&Properties::NvDynamicBoost),
            nv_temp_target: sys_props.contains(&Properties::NvTempTarget),
            panel_od: sys_props.contains(&Properties::PanelOd),
            boot_sound: sys_props.contains(&Properties::PostAnimationSound),
            ppt_apu_sppt: sys_props.contains(&Properties::PptApuSppt),
            ppt_fppt: sys_props.contains(&Properties::PptFppt),
            ppt_pl1_spl: sys_props.contains(&Properties::PptPl1Spl),
            ppt_pl2_sppt: sys_props.contains(&Properties::PptPl2Sppt),
            ppt_platform_sppt: sys_props.contains(&Properties::PptPlatformSppt),
            throttle_thermal_policy: sys_props.contains(&Properties::ThrottlePolicy),
        };

        handle
            .upgrade_in_event_loop(move |handle| {
                handle.global::<SystemPageData>().set_available(props);

                set_ui_callbacks!(handle,
                    SystemPageData(as f32),
                    platform.charge_control_end_threshold(as u8),
                    "Charge limit successfully set to {}",
                    "Setting Charge limit failed"
                );
                set_ui_callbacks!(
                    handle,
                    SystemPageData(),
                    platform.panel_od(),
                    "Panel OverDrive successfully set to {}",
                    "Setting Panel OverDrive failed"
                );
                set_ui_callbacks!(
                    handle,
                    SystemPageData(),
                    platform.boot_sound(),
                    "POST Animation sound successfully set to {}",
                    "Setting POST Animation sound failed"
                );
                set_ui_callbacks!(
                    handle,
                    SystemPageData(),
                    platform.mini_led_mode(),
                    "MiniLED mode successfully set to {}",
                    "Setting MiniLED mode failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as i32),
                    platform.throttle_thermal_policy(.into()),
                    "Throttle policy set to {}",
                    "Setting Throttle policy failed"
                );

                set_ui_callbacks!(handle,
                    SystemPageData(as i32),
                    platform.throttle_balanced_epp(.into()),
                    "Throttle policy EPP set to {}",
                    "Setting Throttle policy EPP failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as i32),
                    platform.throttle_performance_epp(.into()),
                    "Throttle policy EPP set to {}",
                    "Setting Throttle policy EPP failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as i32),
                    platform.throttle_quiet_epp(.into()),
                    "Throttle policy EPP set to {}",
                    "Setting Throttle policy EPP failed"
                );
                set_ui_callbacks!(
                    handle,
                    SystemPageData(),
                    platform.throttle_policy_linked_epp(),
                    "Throttle policy linked to EPP: {}",
                    "Setting Throttle policy linked to EPP failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as i32),
                    platform.throttle_policy_on_ac(.into()),
                    "Throttle policy on AC set to {}",
                    "Setting Throttle policy on AC failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as i32),
                    platform.throttle_policy_on_battery(.into()),
                    "Throttle policy on abttery set to {}",
                    "Setting Throttle policy on battery failed"
                );

                set_ui_callbacks!(handle,
                    SystemPageData(as f32),
                    platform.ppt_pl1_spl(as u8),
                    "ppt_pl1_spl successfully set to {}",
                    "Setting ppt_pl1_spl failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as f32),
                    platform.ppt_pl2_sppt(as u8),
                    "ppt_pl2_sppt successfully set to {}",
                    "Setting ppt_pl2_sppt failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as f32),
                    platform.ppt_fppt(as u8),
                    "ppt_fppt successfully set to {}",
                    "Setting ppt_fppt failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as f32),
                    platform.ppt_apu_sppt(as u8),
                    "ppt_apu_sppt successfully set to {}",
                    "Setting ppt_apu_sppt failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as f32),
                    platform.ppt_platform_sppt(as u8),
                    "ppt_platform_sppt successfully set to {}",
                    "Setting ppt_platform_sppt failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as f32),
                    platform.nv_temp_target(as u8),
                    "nv_temp_target successfully set to {}",
                    "Setting nv_temp_target failed"
                );
                set_ui_callbacks!(handle,
                    SystemPageData(as f32),
                    platform.nv_dynamic_boost(as u8),
                    "nv_dynamic_boost successfully set to {}",
                    "Setting nv_dynamic_boost failed"
                );
            })
            .unwrap();
    });
}
