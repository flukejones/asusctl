use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use egui::Vec2;
use log::error;
use rog_anime::{Animations, DeviceState};
use rog_aura::layouts::KeyLayout;
use rog_aura::usb::AuraPowerDev;
use rog_aura::{AuraEffect, AuraModeNum, LedBrightness};
use rog_platform::platform::{GpuMode, PlatformPolicy};
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::FanCurvePU;
use supergfxctl::pci_device::{GfxMode, GfxPower};
#[cfg(not(feature = "mocking"))]
use supergfxctl::zbus_proxy::DaemonProxyBlocking as GfxProxyBlocking;

use crate::error::Result;
#[cfg(feature = "mocking")]
use crate::mocking::DaemonProxyBlocking as GfxProxyBlocking;
use crate::update_and_notify::EnabledNotifications;
use crate::RogDbusClientBlocking;

#[derive(Clone, Debug, Default)]
pub struct PlatformState {
    /// To be shared to a thread that checks notifications.
    /// It's a bit general in that it won't provide *what* was
    /// updated, so the full state needs refresh
    pub post_sound: Option<bool>,
    pub gpu_mux_mode: Option<GpuMode>,
    pub panel_overdrive: Option<bool>,
    pub mini_led_mode: Option<bool>,
    pub dgpu_disable: Option<bool>,
    pub egpu_enable: Option<bool>,
    pub throttle: Option<PlatformPolicy>,
    pub charge_limit: Option<u8>,
}

impl PlatformState {
    pub fn new(dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        Ok(Self {
            post_sound: dbus.proxies().platform().post_animation_sound().ok(),
            gpu_mux_mode: dbus
                .proxies()
                .platform()
                .gpu_mux_mode()
                .map(GpuMode::from)
                .ok(),
            panel_overdrive: dbus.proxies().platform().panel_od().ok(),
            mini_led_mode: dbus.proxies().platform().mini_led_mode().ok(),
            // TODO: needs supergfx
            dgpu_disable: dbus.proxies().platform().dgpu_disable().ok(),
            egpu_enable: dbus.proxies().platform().egpu_enable().ok(),
            throttle: dbus.proxies().platform().throttle_thermal_policy().ok(),
            charge_limit: dbus
                .proxies()
                .platform()
                .charge_control_end_threshold()
                .ok(),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct FanCurvesState {
    pub show_curve: PlatformPolicy,
    pub show_graph: FanCurvePU,
    pub curves: BTreeMap<PlatformPolicy, Vec<CurveData>>,
    pub available_fans: HashSet<FanCurvePU>,
    pub drag_delta: Vec2,
}

impl FanCurvesState {
    pub fn new(dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        let profiles = vec![
            PlatformPolicy::Balanced,
            PlatformPolicy::Quiet,
            PlatformPolicy::Performance,
        ];

        let mut curves: BTreeMap<PlatformPolicy, Vec<CurveData>> = BTreeMap::new();
        for p in &profiles {
            if let Ok(curve) = dbus.proxies().fan_curves().fan_curve_data(*p) {
                curves.insert(*p, curve);
            } else {
                curves.insert(*p, Default::default());
            }
        }

        let available_fans = HashSet::new();
        // for fan in supported.platform_profile.fans.iter() {
        //     available_fans.insert(*fan);
        // }

        let show_curve = dbus.proxies().platform().throttle_thermal_policy()?;

        Ok(Self {
            show_curve,
            show_graph: FanCurvePU::CPU,
            curves,
            available_fans,
            drag_delta: Vec2::default(),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct AuraState {
    pub current_mode: AuraModeNum,
    pub modes: BTreeMap<AuraModeNum, AuraEffect>,
    pub enabled: AuraPowerDev,
    /// Brightness from 0-3
    pub bright: LedBrightness,
    pub wave_red: [u8; 22],
    pub wave_green: [u8; 22],
    pub wave_blue: [u8; 22],
}

impl AuraState {
    pub fn new(layout: &KeyLayout, dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        Ok(Self {
            current_mode: if !layout.basic_modes().is_empty() {
                dbus.proxies().aura().led_mode().unwrap_or_default()
            } else {
                AuraModeNum::Static
            },

            modes: if !layout.basic_modes().is_empty() {
                dbus.proxies().aura().all_mode_data().unwrap_or_default()
            } else {
                BTreeMap::new()
            },
            enabled: dbus.proxies().aura().led_power().unwrap_or_default(),
            bright: dbus.proxies().aura().brightness().unwrap_or_default(),
            wave_red: [0u8; 22],
            wave_green: [0u8; 22],
            wave_blue: [0u8; 22],
        })
    }

    /// Bump value in to the wave and surf all along.
    pub fn nudge_wave(&mut self, r: u8, g: u8, b: u8) {
        for i in (0..self.wave_red.len()).rev() {
            if i > 0 {
                self.wave_red[i] = self.wave_red[i - 1];
                self.wave_green[i] = self.wave_green[i - 1];
                self.wave_blue[i] = self.wave_blue[i - 1];
            }
        }
        self.wave_red[0] = r;
        self.wave_green[0] = g;
        self.wave_blue[0] = b;
    }
}

#[derive(Clone, Debug, Default)]
pub struct AnimeState {
    pub display_enabled: bool,
    pub display_brightness: u8,
    pub builtin_anims_enabled: bool,
    pub builtin_anims: Animations,
}

impl AnimeState {
    pub fn new(dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        let device_state = dbus.proxies().anime().device_state()?;
        Ok(Self {
            display_enabled: device_state.display_enabled,
            display_brightness: device_state.display_brightness as u8,
            builtin_anims_enabled: device_state.builtin_anims_enabled,
            builtin_anims: device_state.builtin_anims,
        })
    }
}

impl From<DeviceState> for AnimeState {
    fn from(dev: DeviceState) -> Self {
        Self {
            display_enabled: dev.display_enabled,
            display_brightness: dev.display_brightness as u8,
            builtin_anims_enabled: dev.builtin_anims_enabled,
            builtin_anims: dev.builtin_anims,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GfxState {
    pub has_supergfx: bool,
    pub mode: GfxMode,
    pub power_status: GfxPower,
}

impl GfxState {
    pub fn new(dbus: &GfxProxyBlocking<'_>) -> Result<Self> {
        Ok(Self {
            has_supergfx: dbus.mode().is_ok(),
            mode: dbus.mode().unwrap_or(GfxMode::None),
            power_status: dbus.power().unwrap_or(GfxPower::Unknown),
        })
    }
}

impl Default for GfxState {
    fn default() -> Self {
        Self {
            has_supergfx: false,
            mode: GfxMode::None,
            power_status: GfxPower::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuraCreation {
    /// Specifically for testing the development of keyboard layouts (combined
    /// with `--layout-name` CLI option)
    pub layout_testing: Option<PathBuf>,
    pub layout_last_modified: SystemTime,
    pub keyboard_layout: KeyLayout,
    pub keyboard_layouts: Vec<PathBuf>,
    /// current index in to `self.keyboard_layouts`
    pub keyboard_layout_index: usize,
}

impl AuraCreation {
    pub fn new(
        layout_testing: Option<PathBuf>,
        keyboard_layout: KeyLayout,
        keyboard_layouts: Vec<PathBuf>,
    ) -> Self {
        Self {
            layout_testing,
            layout_last_modified: SystemTime::now(),
            keyboard_layout,
            keyboard_layouts,
            keyboard_layout_index: 0,
        }
    }
}

///  State stored from system daemons. This is shared with: tray, zbus
/// notifications thread and the GUI app thread.
pub struct SystemState {
    pub aura_creation: AuraCreation,
    //--
    pub enabled_notifications: Arc<Mutex<EnabledNotifications>>,
    /// Because much of the app state here is the same as
    /// `RogBiosSupportedFunctions` we can re-use that structure.
    pub bios: PlatformState,
    pub aura: AuraState,
    pub anime: AnimeState,
    pub fan_curves: FanCurvesState,
    pub gfx_state: GfxState,
    pub error: Option<String>,
    /// Specific field for the tray only so that we can know when it does need
    /// update. The tray should set this to false when done.
    pub tray_should_update: bool,
    pub app_should_update: bool,
    pub asus_dbus: RogDbusClientBlocking<'static>,
    pub gfx_dbus: GfxProxyBlocking<'static>,
    pub tray_enabled: bool,
    pub run_in_bg: bool,
}

impl SystemState {
    /// Creates self, including the relevant dbus connections and proixies for
    /// internal use
    pub fn new(
        layout_testing: Option<PathBuf>,
        keyboard_layout: KeyLayout,
        keyboard_layouts: Vec<PathBuf>,
        enabled_notifications: Arc<Mutex<EnabledNotifications>>,
        tray_enabled: bool,
        run_in_bg: bool,
    ) -> Result<Self> {
        let (asus_dbus, conn) = RogDbusClientBlocking::new()?;
        let gfx_dbus = GfxProxyBlocking::new(&conn).expect("Couldn't connect to supergfxd");

        let aura = AuraState::new(&keyboard_layout, &asus_dbus)
            .map_err(|e| {
                let e = format!("Could not get AuraState state: {e}");
                error!("{e}");
            })
            .unwrap_or_default();

        Ok(Self {
            aura_creation: AuraCreation::new(layout_testing, keyboard_layout, keyboard_layouts),
            enabled_notifications,
            bios: PlatformState::new(&asus_dbus)
                .map_err(|e| {
                    let e = format!("Could not get BiosState state: {e}");
                    error!("{e}");
                })
                .unwrap_or_default(),
            aura,
            anime: AnimeState::new(&asus_dbus)
                .map_err(|e| {
                    let e = format!("Could not get AanimeState state: {e}");
                    error!("{e}");
                })
                .unwrap_or_default(),
            fan_curves: FanCurvesState::new(&asus_dbus)
                .map_err(|e| {
                    let e = format!("Could not get FanCurvesState state: {e}");
                    error!("{e}");
                })
                .unwrap_or_default(),
            gfx_state: GfxState::new(&gfx_dbus)
                .map_err(|e| {
                    let e = format!("Could not get supergfxd state: {e}");
                    error!("{e}");
                })
                .unwrap_or_default(),
            error: None,
            tray_should_update: true,
            app_should_update: true,
            asus_dbus,
            gfx_dbus,
            tray_enabled,
            run_in_bg,
        })
    }

    pub fn set_notified(&mut self) {
        self.tray_should_update = true;
        self.app_should_update = true;
    }
}

impl Default for SystemState {
    fn default() -> Self {
        let (asus_dbus, conn) = RogDbusClientBlocking::new().expect("Couldn't connect to asusd");
        let gfx_dbus = GfxProxyBlocking::new(&conn).expect("Couldn't connect to supergfxd");

        Self {
            aura_creation: AuraCreation {
                layout_testing: None,
                layout_last_modified: SystemTime::now(),
                keyboard_layout: KeyLayout::default_layout(),
                keyboard_layouts: Default::default(),
                keyboard_layout_index: 0,
            },
            enabled_notifications: Default::default(),
            bios: PlatformState {
                post_sound: Default::default(),
                gpu_mux_mode: None,
                charge_limit: Some(100),
                ..Default::default()
            },
            aura: AuraState {
                current_mode: AuraModeNum::Static,
                modes: Default::default(),
                enabled: AuraPowerDev::default(),
                ..Default::default()
            },
            anime: AnimeState::default(),
            fan_curves: FanCurvesState {
                ..Default::default()
            },
            gfx_state: GfxState {
                has_supergfx: false,
                mode: GfxMode::None,
                power_status: GfxPower::Unknown,
            },
            error: Default::default(),
            tray_should_update: true,
            app_should_update: true,
            asus_dbus,
            gfx_dbus,
            tray_enabled: true,
            run_in_bg: true,
        }
    }
}
