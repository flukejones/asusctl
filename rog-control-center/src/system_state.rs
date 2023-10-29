use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use egui::Vec2;
use log::error;
use rog_anime::{Animations, DeviceState};
use rog_aura::layouts::KeyLayout;
use rog_aura::usb::AuraPowerDev;
use rog_aura::{AuraEffect, AuraModeNum};
use rog_platform::platform::GpuMode;
use rog_platform::supported::SupportedFunctions;
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::{FanCurvePU, Profile};
use supergfxctl::pci_device::{GfxMode, GfxPower};
#[cfg(not(feature = "mocking"))]
use supergfxctl::zbus_proxy::DaemonProxyBlocking as GfxProxyBlocking;

use crate::error::Result;
#[cfg(feature = "mocking")]
use crate::mocking::DaemonProxyBlocking as GfxProxyBlocking;
use crate::update_and_notify::EnabledNotifications;
use crate::RogDbusClientBlocking;

#[derive(Clone, Debug, Default)]
pub struct BiosState {
    /// To be shared to a thread that checks notifications.
    /// It's a bit general in that it won't provide *what* was
    /// updated, so the full state needs refresh
    pub post_sound: bool,
    pub dedicated_gfx: GpuMode,
    pub panel_overdrive: bool,
    pub mini_led_mode: bool,
    pub dgpu_disable: bool,
    pub egpu_enable: bool,
}

impl BiosState {
    pub fn new(supported: &SupportedFunctions, dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        Ok(Self {
            post_sound: if supported.rog_bios_ctrl.post_sound {
                dbus.proxies().rog_bios().post_boot_sound()? != 0
            } else {
                false
            },
            dedicated_gfx: if supported.rog_bios_ctrl.gpu_mux {
                dbus.proxies().rog_bios().gpu_mux_mode()?
            } else {
                GpuMode::NotSupported
            },
            panel_overdrive: if supported.rog_bios_ctrl.panel_overdrive {
                dbus.proxies().rog_bios().panel_od()?
            } else {
                false
            },
            mini_led_mode: if supported.rog_bios_ctrl.mini_led_mode {
                dbus.proxies().rog_bios().mini_led_mode()?
            } else {
                false
            },
            // TODO: needs supergfx
            dgpu_disable: supported.rog_bios_ctrl.dgpu_disable,
            egpu_enable: supported.rog_bios_ctrl.egpu_enable,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProfilesState {
    pub list: Vec<Profile>,
    pub current: Profile,
}

impl ProfilesState {
    pub fn new(supported: &SupportedFunctions, dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        Ok(Self {
            list: if supported.platform_profile.platform_profile {
                let mut list = dbus.proxies().profile().profiles()?;
                list.sort();
                list
            } else {
                vec![]
            },
            current: if supported.platform_profile.platform_profile {
                dbus.proxies().profile().active_profile()?
            } else {
                Profile::Balanced
            },
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct FanCurvesState {
    pub show_curve: Profile,
    pub show_graph: FanCurvePU,
    pub curves: BTreeMap<Profile, Vec<CurveData>>,
    pub available_fans: HashSet<FanCurvePU>,
    pub drag_delta: Vec2,
}

impl FanCurvesState {
    pub fn new(supported: &SupportedFunctions, dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        let profiles = if supported.platform_profile.platform_profile {
            dbus.proxies().profile().profiles()?
        } else {
            vec![Profile::Balanced, Profile::Quiet, Profile::Performance]
        };

        let mut curves: BTreeMap<Profile, Vec<CurveData>> = BTreeMap::new();
        for p in &profiles {
            if !supported.platform_profile.fans.is_empty() {
                if let Ok(curve) = dbus.proxies().profile().fan_curve_data(*p) {
                    curves.insert(*p, curve);
                }
            } else {
                curves.insert(*p, Vec::default());
            }
        }

        let mut available_fans = HashSet::new();
        for fan in supported.platform_profile.fans.iter() {
            available_fans.insert(*fan);
        }

        let show_curve = if !supported.platform_profile.fans.is_empty() {
            dbus.proxies().profile().active_profile()?
        } else {
            Profile::Balanced
        };

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
    pub bright: i16,
    pub wave_red: [u8; 22],
    pub wave_green: [u8; 22],
    pub wave_blue: [u8; 22],
}

impl AuraState {
    pub fn new(layout: &KeyLayout, dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        Ok(Self {
            current_mode: if !layout.basic_modes().is_empty() {
                dbus.proxies().led().led_mode().unwrap_or_default()
            } else {
                AuraModeNum::Static
            },

            modes: if !layout.basic_modes().is_empty() {
                dbus.proxies().led().led_modes().unwrap_or_default()
            } else {
                BTreeMap::new()
            },
            enabled: dbus.proxies().led().led_power().unwrap_or_default(),
            bright: dbus.proxies().led().led_brightness().unwrap_or_default(),
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
    pub fn new(supported: &SupportedFunctions, dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        if supported.anime_ctrl.0 {
            let device_state = dbus.proxies().anime().device_state()?;
            Ok(Self {
                display_enabled: device_state.display_enabled,
                display_brightness: device_state.display_brightness as u8,
                builtin_anims_enabled: device_state.builtin_anims_enabled,
                builtin_anims: device_state.builtin_anims,
            })
        } else {
            Ok(Default::default())
        }
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
    pub fn new(_supported: &SupportedFunctions, dbus: &GfxProxyBlocking<'_>) -> Result<Self> {
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

#[derive(Clone, Debug, Default)]
pub struct PowerState {
    pub charge_limit: u8,
    pub ac_power: bool,
}

impl PowerState {
    pub fn new(_supported: &SupportedFunctions, dbus: &RogDbusClientBlocking<'_>) -> Result<Self> {
        Ok(Self {
            charge_limit: dbus.proxies().charge().charge_control_end_threshold()?,
            ac_power: dbus.proxies().charge().mains_online()?,
        })
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
    pub bios: BiosState,
    pub aura: AuraState,
    pub anime: AnimeState,
    pub profiles: ProfilesState,
    pub fan_curves: FanCurvesState,
    pub gfx_state: GfxState,
    pub power_state: PowerState,
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
        supported: &SupportedFunctions,
    ) -> Result<Self> {
        let (asus_dbus, conn) = RogDbusClientBlocking::new()?;
        let mut error = None;
        let gfx_dbus = GfxProxyBlocking::new(&conn).expect("Couldn't connect to supergfxd");
        let aura = AuraState::new(&keyboard_layout, &asus_dbus)
            .map_err(|e| {
                let e = format!("Could not get AuraState state: {e}");
                error!("{e}");
                error = Some(e);
            })
            .unwrap_or_default();

        Ok(Self {
            aura_creation: AuraCreation::new(layout_testing, keyboard_layout, keyboard_layouts),
            enabled_notifications,
            power_state: PowerState::new(supported, &asus_dbus)
                .map_err(|e| {
                    let e = format!("Could not get PowerState state: {e}");
                    error!("{e}");
                    error = Some(e);
                })
                .unwrap_or_default(),
            bios: BiosState::new(supported, &asus_dbus)
                .map_err(|e| {
                    let e = format!("Could not get BiosState state: {e}");
                    error!("{e}");
                    error = Some(e);
                })
                .unwrap_or_default(),
            aura,
            anime: AnimeState::new(supported, &asus_dbus)
                .map_err(|e| {
                    let e = format!("Could not get AanimeState state: {e}");
                    error!("{e}");
                    error = Some(e);
                })
                .unwrap_or_default(),
            profiles: ProfilesState::new(supported, &asus_dbus)
                .map_err(|e| {
                    let e = format!("Could not get ProfilesState state: {e}");
                    error!("{e}");
                    error = Some(e);
                })
                .unwrap_or_default(),
            fan_curves: FanCurvesState::new(supported, &asus_dbus)
                .map_err(|e| {
                    let e = format!("Could not get FanCurvesState state: {e}");
                    error!("{e}");
                    error = Some(e);
                })
                .unwrap_or_default(),
            gfx_state: GfxState::new(supported, &gfx_dbus)
                .map_err(|e| {
                    let e = format!("Could not get supergfxd state: {e}");
                    error!("{e}");
                    error = Some(e);
                })
                .unwrap_or_default(),
            error,
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
            bios: BiosState {
                post_sound: Default::default(),
                dedicated_gfx: GpuMode::NotSupported,
                ..Default::default()
            },
            aura: AuraState {
                current_mode: AuraModeNum::Static,
                modes: Default::default(),
                enabled: AuraPowerDev::default(),
                ..Default::default()
            },
            anime: AnimeState::default(),
            profiles: ProfilesState {
                ..Default::default()
            },
            fan_curves: FanCurvesState {
                ..Default::default()
            },
            gfx_state: GfxState {
                has_supergfx: false,
                mode: GfxMode::None,
                power_status: GfxPower::Unknown,
            },
            power_state: PowerState {
                charge_limit: 99,
                ac_power: false,
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
