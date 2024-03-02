use std::sync::{Arc, Mutex};

use config_traits::StdConfig;
use rog_anime::Animations;
use rog_dbus::zbus_anime::AnimeProxy;
use rog_dbus::zbus_aura::AuraProxy;
use rog_dbus::zbus_platform::{PlatformProxy, PlatformProxyBlocking};
use rog_platform::platform::Properties;
use slint::{Color, ComponentHandle, Model, RgbaColor, SharedString, Weak};
use zbus::proxy::CacheProperties;

use crate::config::Config;
use crate::{
    AnimePageData, AppSettingsPageData, AuraPageData, AvailableSystemProperties, MainWindow,
    SystemPageData,
};

impl From<rog_aura::AuraEffect> for crate::slint_generatedMainWindow::AuraEffect {
    fn from(m: rog_aura::AuraEffect) -> Self {
        Self {
            colour1: RgbaColor {
                red: m.colour1.r,
                green: m.colour1.g,
                blue: m.colour1.b,
                alpha: 255,
            }
            .into(),
            colour2: RgbaColor {
                red: m.colour2.r,
                green: m.colour2.g,
                blue: m.colour2.b,
                alpha: 255,
            }
            .into(),
            direction: m.direction.into(),
            mode: m.mode.into(),
            speed: m.speed.into(),
            zone: m.zone.into(),
        }
    }
}

impl From<crate::slint_generatedMainWindow::AuraEffect> for rog_aura::AuraEffect {
    fn from(m: crate::slint_generatedMainWindow::AuraEffect) -> Self {
        let c1: RgbaColor<u8> = m.colour1.into();
        let c2: RgbaColor<u8> = m.colour2.into();
        Self {
            colour1: rog_aura::Colour {
                r: c1.red,
                g: c1.green,
                b: c1.blue,
            },
            colour2: rog_aura::Colour {
                r: c2.red,
                g: c2.green,
                b: c2.blue,
            },
            direction: m.direction.into(),
            mode: m.mode.into(),
            speed: m.speed.into(),
            zone: m.zone.into(),
        }
    }
}

// This macro expects are consistent naming between proxy calls and slint
// globals
macro_rules! set_ui_props_async {
    ($ui:ident, $proxy:ident, $global:ident, $proxy_fn:ident) => {
        if let Ok(value) = $proxy.$proxy_fn().await {
            $ui.upgrade_in_event_loop(move |handle| {
                concat_idents::concat_idents!(set = set_, $proxy_fn {
                    handle.global::<$global>().set(value.into());
                });
            }).ok();
        }
    };
}

// this macro sets up:
// - a link from UI callback -> dbus proxy property
// - a link from dbus property signal -> UI state
// conv1 and conv2 are type conversion args
macro_rules! set_ui_callbacks {
    ($handle:ident, $data:ident($($conv1: tt)*),$proxy:ident.$proxy_fn:tt($($conv2: tt)*),$success:literal,$failed:literal) => {
        let handle_copy = $handle.as_weak();
        let proxy_copy = $proxy.clone();
        let data = $handle.global::<$data>();
        concat_idents::concat_idents!(on_set = on_set_, $proxy_fn {
        data.on_set(move |value| {
            let proxy_copy = proxy_copy.clone();
            let handle_copy = handle_copy.clone();
            tokio::spawn(async move {
                concat_idents::concat_idents!(set = set_, $proxy_fn {
                show_toast(
                    format!($success, value).into(),
                    $failed.into(),
                    handle_copy,
                    proxy_copy.set(value $($conv2)*).await,
                );
                });
            });
            });
        });
        let handle_copy = $handle.as_weak();
        let proxy_copy = $proxy.clone();
        concat_idents::concat_idents!(receive = receive_, $proxy_fn, _changed {
        // spawn required since the while let never exits
        tokio::spawn(async move {
            let mut x = proxy_copy.receive().await;
            concat_idents::concat_idents!(set = set_, $proxy_fn {
            use zbus::export::futures_util::StreamExt;
            while let Some(e) = x.next().await {
                if let Ok(out) = e.get().await {
                    handle_copy.upgrade_in_event_loop(move |handle| {
                        handle.global::<$data>().set(out $($conv1)*);
                    }).ok();
                }
            }
            });
        });
        });
    };
}

pub fn setup_window(_config: Arc<Mutex<Config>>) -> MainWindow {
    let ui = MainWindow::new().unwrap();

    let conn = zbus::blocking::Connection::system().unwrap();
    let platform = PlatformProxyBlocking::new(&conn).unwrap();

    let interfaces = platform.supported_interfaces().unwrap();
    log::debug!("Available interfaces: {interfaces:?}");
    // "Anime", "Aura", "FanCurves", "Platform"
    ui.set_sidebar_items_avilable(
        [
            // Needs to match the order of slint sidebar items
            interfaces.contains(&"Platform".into()),
            interfaces.contains(&"Aura".into()),
            interfaces.contains(&"Anime".into()),
            interfaces.contains(&"FanCurves".into()),
            true,
            true,
        ]
        .into(),
    );

    ui.on_exit_app(move || {
        slint::quit_event_loop().unwrap();
    });

    setup_app_settings_page(&ui, _config.clone());
    setup_system_page(&ui, _config.clone());
    setup_system_page_callbacks(&ui, _config.clone());
    setup_aura_page(&ui, _config.clone());
    setup_anime_page(&ui, _config);

    ui
}

pub fn setup_app_settings_page(ui: &MainWindow, config: Arc<Mutex<Config>>) {
    let config_copy = config.clone();
    let global = ui.global::<AppSettingsPageData>();
    global.on_set_run_in_background(move |enable| {
        if let Ok(mut lock) = config_copy.try_lock() {
            lock.run_in_background = enable;
            lock.write();
        }
    });
    let config_copy = config.clone();
    global.on_set_startup_in_background(move |enable| {
        if let Ok(mut lock) = config_copy.try_lock() {
            lock.startup_in_background = enable;
            lock.write();
        }
    });
    let config_copy = config.clone();
    global.on_set_enable_tray_icon(move |enable| {
        if let Ok(mut lock) = config_copy.try_lock() {
            lock.enable_tray_icon = enable;
            lock.write();
        }
    });
    let config_copy = config.clone();
    global.on_set_enable_notifications(move |enable| {
        if let Ok(mut lock) = config_copy.try_lock() {
            lock.enable_notifications = enable;
            lock.write();
        }
    });

    if let Ok(lock) = config.try_lock() {
        global.set_run_in_background(lock.run_in_background);
        global.set_startup_in_background(lock.startup_in_background);
        global.set_enable_tray_icon(lock.enable_tray_icon);
        global.set_enable_notifications(lock.enable_notifications);
    }
}

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

fn decode_hex(s: &str) -> RgbaColor<u8> {
    let s = s.trim_start_matches('#');
    let c: Vec<u8> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or(164))
        .collect();
    RgbaColor {
        alpha: 255,
        red: *c.first().unwrap_or(&255),
        green: *c.get(1).unwrap_or(&128),
        blue: *c.get(2).unwrap_or(&32),
    }
}

fn rgb_hi(colour: Color) -> (f32, f32) {
    let c1: RgbaColor<f32> = RgbaColor::from(colour);
    let r = c1.red / 255.0;
    let g = c1.green / 255.0;
    let b = c1.blue / 255.0;

    let min = r.min(g.min(b));
    let max = r.max(g.max(b));
    let delta = max - min;

    let h = match delta == 0.0 {
        true => 0.0,
        false => {
            if r == max {
                (g - b) / delta
            } else if g == max {
                2.0 + (b - r) / delta
            } else {
                4.0 + (r - g) / delta
            }
        }
    };
    let h2 = ((h * 60.0) + 360.0) % 360.0;
    let i = 0.299 * c1.red + 0.587 * c1.green + 0.114 * c1.blue;

    (h2, i)
}

fn setup_aura_page(ui: &MainWindow, _states: Arc<Mutex<Config>>) {
    ui.global::<AuraPageData>().on_blend_colour(|c1, c2, f| {
        let c1: RgbaColor<f32> = RgbaColor::from(c1);
        let c2: RgbaColor<f32> = RgbaColor::from(c2);

        let c1 = RgbaColor {
            alpha: 1.0,
            red: c1.red + (c2.red - c1.red) * f,
            green: c1.green + (c2.green - c1.green) * f,
            blue: c1.blue + (c2.blue - c1.blue) * f,
        };
        c1.into()
    });

    ui.global::<AuraPageData>().on_blend_lightness(|c1, f| {
        let c1: RgbaColor<f32> = RgbaColor::from(c1);
        let c = RgbaColor {
            alpha: 1.0,
            red: c1.red * f,
            green: c1.green * f,
            blue: c1.blue * f,
        };
        // dbg!(rgb_hi(c.into()));
        c.into()
    });

    ui.global::<AuraPageData>().on_set_hex_from_colour(|c| {
        format!("#{:02X}{:02X}{:02X}", c.red(), c.green(), c.blue()).into()
    });

    ui.global::<AuraPageData>()
        .on_set_hex_to_colour(|s| decode_hex(s.as_str()).into());

    ui.global::<AuraPageData>().on_set_hue(|c| rgb_hi(c).0);
    ui.global::<AuraPageData>().on_set_bright(|c| rgb_hi(c).1);

    let handle = ui.as_weak();
    tokio::spawn(async move {
        let conn = zbus::Connection::system().await.unwrap();
        let aura = AuraProxy::builder(&conn)
            .cache_properties(CacheProperties::Yes)
            .build()
            .await
            .unwrap();

        set_ui_props_async!(handle, aura, AuraPageData, brightness);
        set_ui_props_async!(handle, aura, AuraPageData, led_mode);
        set_ui_props_async!(handle, aura, AuraPageData, led_mode_data);

        if let Ok(modes) = aura.supported_basic_modes().await {
            log::debug!("Available LED modes {modes:?}");
            handle
                .upgrade_in_event_loop(move |handle| {
                    let m: Vec<i32> = modes.iter().map(|n| (*n).into()).collect();
                    handle
                        .global::<AuraPageData>()
                        .set_supported_basic_modes(m.as_slice().into());
                    // Get the translated names
                    let names = handle.global::<AuraPageData>().get_mode_names();

                    let res: Vec<SharedString> = names
                        .iter()
                        .enumerate()
                        .filter(|(n, _)| modes.contains(&(*n as i32).into()) && *n != 9)
                        .map(|(_, i)| i)
                        .collect();
                    handle
                        .global::<AuraPageData>()
                        .set_available_mode_names(res.as_slice().into());
                })
                .ok();
        }

        let proxy_copy = aura.clone();
        handle
            .upgrade_in_event_loop(move |handle| {
                set_ui_callbacks!(handle,
                    AuraPageData(.into()),
                    aura.brightness(.into()),
                    "Keyboard LED brightness successfully set to {}",
                    "Setting keyboard LED brightness failed"
                );

                set_ui_callbacks!(handle,
                    AuraPageData(.into()),
                    aura.led_mode(.into()),
                    "Keyboard LED mode successfully set to {}",
                    "Setting keyboard LEDmode failed"
                );

                set_ui_callbacks!(handle,
                    AuraPageData(.into()),
                    aura.led_mode_data(.into()),
                    "Keyboard LED mode set to {:?}",
                    "Setting keyboard LED mode failed"
                );
            })
            .ok();

        // Need to update the UI if the mode changes
        let handle_copy = handle.clone();
        // spawn required since the while let never exits
        tokio::spawn(async move {
            let mut x = proxy_copy.receive_led_mode_data_changed().await;
            use zbus::export::futures_util::StreamExt;
            while let Some(e) = x.next().await {
                if let Ok(out) = e.get().await {
                    handle_copy
                        .upgrade_in_event_loop(move |handle| {
                            handle
                                .global::<AuraPageData>()
                                .invoke_update_led_mode_data(out.into());
                        })
                        .ok();
                }
            }
        });
    });
}

fn setup_anime_page(ui: &MainWindow, _states: Arc<Mutex<Config>>) {
    let handle = ui.as_weak();
    tokio::spawn(async move {
        let conn = zbus::Connection::system().await.unwrap();
        let anime = AnimeProxy::new(&conn).await.unwrap();

        set_ui_props_async!(handle, anime, AnimePageData, brightness);
        set_ui_props_async!(handle, anime, AnimePageData, builtins_enabled);
        set_ui_props_async!(handle, anime, AnimePageData, enable_display);
        set_ui_props_async!(handle, anime, AnimePageData, off_when_lid_closed);
        set_ui_props_async!(handle, anime, AnimePageData, off_when_suspended);
        set_ui_props_async!(handle, anime, AnimePageData, off_when_unplugged);

        let builtins = anime.builtin_animations().await.unwrap_or_default();
        handle
            .upgrade_in_event_loop(move |handle| {
                {
                    let global = handle.global::<AnimePageData>();
                    global.set_boot_anim(builtins.boot as i32);
                    global.set_awake_anim(builtins.awake as i32);
                    global.set_sleep_anim(builtins.sleep as i32);
                    global.set_shutdown_anim(builtins.shutdown as i32);

                    let handle_copy = handle.as_weak();
                    let anime_copy = anime.clone();
                    global.on_set_builtin_animations(move |boot, awake, sleep, shutdown| {
                        let handle_copy = handle_copy.clone();
                        let anime_copy = anime_copy.clone();
                        tokio::spawn(async move {
                            show_toast(
                                "Anime builtin animations changed".into(),
                                "Failed to set Anime builtin animations".into(),
                                handle_copy,
                                anime_copy
                                    .set_builtin_animations(Animations {
                                        boot: boot.into(),
                                        awake: awake.into(),
                                        sleep: sleep.into(),
                                        shutdown: shutdown.into(),
                                    })
                                    .await,
                            );
                        });
                    });

                    let handle_copy = handle.as_weak();
                    let anime_copy = anime.clone();
                    tokio::spawn(async move {
                        let mut x = anime_copy.receive_builtin_animations_changed().await;
                        use zbus::export::futures_util::StreamExt;
                        while let Some(e) = x.next().await {
                            if let Ok(out) = e.get().await {
                                handle_copy
                                    .upgrade_in_event_loop(move |handle| {
                                        handle
                                            .global::<AnimePageData>()
                                            .set_boot_anim(out.boot.into());
                                        handle
                                            .global::<AnimePageData>()
                                            .set_awake_anim(out.awake.into());
                                        handle
                                            .global::<AnimePageData>()
                                            .set_sleep_anim(out.sleep.into());
                                        handle
                                            .global::<AnimePageData>()
                                            .set_shutdown_anim(out.shutdown.into());
                                    })
                                    .ok();
                            }
                        }
                    });
                }

                set_ui_callbacks!(handle,
                    AnimePageData(.into()),
                    anime.brightness(.into()),
                    "Anime LED brightness successfully set to {}",
                    "Setting Anime LED brightness failed"
                );
                set_ui_callbacks!(
                    handle,
                    AnimePageData(),
                    anime.builtins_enabled(),
                    "Keyboard LED mode successfully set to {}",
                    "Setting keyboard LEDmode failed"
                );
                set_ui_callbacks!(
                    handle,
                    AnimePageData(),
                    anime.enable_display(),
                    "Anime display successfully set to {}",
                    "Setting Anime display failed"
                );
                set_ui_callbacks!(
                    handle,
                    AnimePageData(),
                    anime.off_when_lid_closed(),
                    "Anime off_when_lid_closed successfully set to {}",
                    "Setting Anime off_when_lid_closed failed"
                );
                set_ui_callbacks!(
                    handle,
                    AnimePageData(),
                    anime.off_when_suspended(),
                    "Anime off_when_suspended successfully set to {}",
                    "Setting Anime off_when_suspended failed"
                );
                set_ui_callbacks!(
                    handle,
                    AnimePageData(),
                    anime.off_when_unplugged(),
                    "Anime off_when_unplugged successfully set to {}",
                    "Setting Anime off_when_unplugged failed"
                );
            })
            .unwrap();
    });
}

fn show_toast(
    success: SharedString,
    fail: SharedString,
    handle: Weak<MainWindow>,
    result: zbus::Result<()>,
) {
    match result {
        Ok(_) => {
            slint::invoke_from_event_loop(move || handle.unwrap().invoke_show_toast(success)).ok()
        }
        Err(e) => slint::invoke_from_event_loop(move || {
            log::warn!("{fail}: {e}");
            handle.unwrap().invoke_show_toast(fail)
        })
        .ok(),
    };
}
