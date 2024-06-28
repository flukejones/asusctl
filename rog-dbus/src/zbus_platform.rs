//! # `DBus` interface proxy for: `org.asuslinux.Daemon`
//!
//! This code was generated by `zbus-xmlgen` `1.0.0` from `DBus` introspection
//! data. Source: `Interface '/org/asuslinux/Platform' from service
//! 'org.asuslinux.Daemon' on system bus`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the
//! [Writing a client proxy](https://zeenix.pages.freedesktop.org/zbus/client.html)
//! section of the zbus documentation.
//!
//! This `DBus` object implements
//! [standard `DBus` interfaces](https://dbus.freedesktop.org/doc/dbus-specification.html),
//! (`org.freedesktop.DBus.*`) for which the following zbus proxies can be used:
//!
//! * [`zbus::fdo::PropertiesProxy`]
//! * [`zbus::fdo::PeerProxy`]
//! * [`zbus::fdo::IntrospectableProxy`]
//!
//! …consequently `zbus-xmlgen` did not generate code for the above interfaces.

use rog_platform::cpu::CPUEPP;
use rog_platform::platform::{GpuMode, Properties, ThrottlePolicy};
use zbus::proxy;

#[proxy(
    interface = "org.asuslinux.Platform",
    default_service = "org.asuslinux.Daemon",
    default_path = "/org/asuslinux"
)]
trait Platform {
    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;

    /// NextThrottleThermalPolicy method
    fn next_throttle_thermal_policy(&self) -> zbus::Result<()>;

    /// SupportedProperties method
    fn supported_properties(&self) -> zbus::Result<Vec<Properties>>;

    /// ChargeControlEndThreshold property
    #[zbus(property)]
    fn charge_control_end_threshold(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_charge_control_end_threshold(&self, value: u8) -> zbus::Result<()>;

    /// DgpuDisable property
    #[zbus(property)]
    fn dgpu_disable(&self) -> zbus::Result<bool>;

    /// EgpuEnable property
    #[zbus(property)]
    fn egpu_enable(&self) -> zbus::Result<bool>;

    /// GpuMuxMode property
    #[zbus(property)]
    fn gpu_mux_mode(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_gpu_mux_mode(&self, value: GpuMode) -> zbus::Result<()>;

    /// MiniLedMode property
    #[zbus(property)]
    fn mini_led_mode(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_mini_led_mode(&self, value: bool) -> zbus::Result<()>;

    /// NvDynamicBoost property
    #[zbus(property)]
    fn nv_dynamic_boost(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_nv_dynamic_boost(&self, value: u8) -> zbus::Result<()>;

    /// NvTempTarget property
    #[zbus(property)]
    fn nv_temp_target(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_nv_temp_target(&self, value: u8) -> zbus::Result<()>;

    /// PanelOd property
    #[zbus(property)]
    fn panel_od(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_panel_od(&self, value: bool) -> zbus::Result<()>;

    /// PostAnimationSound property
    #[zbus(property)]
    fn boot_sound(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_boot_sound(&self, value: bool) -> zbus::Result<()>;

    /// PptApuSppt property
    #[zbus(property)]
    fn ppt_apu_sppt(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_ppt_apu_sppt(&self, value: u8) -> zbus::Result<()>;

    /// PptFppt property
    #[zbus(property)]
    fn ppt_fppt(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_ppt_fppt(&self, value: u8) -> zbus::Result<()>;

    /// PptPl1Spl property
    #[zbus(property)]
    fn ppt_pl1_spl(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_ppt_pl1_spl(&self, value: u8) -> zbus::Result<()>;

    /// PptPl2Sppt property
    #[zbus(property)]
    fn ppt_pl2_sppt(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_ppt_pl2_sppt(&self, value: u8) -> zbus::Result<()>;

    /// PptPlatformSppt property
    #[zbus(property)]
    fn ppt_platform_sppt(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn set_ppt_platform_sppt(&self, value: u8) -> zbus::Result<()>;

    /// ThrottleBalancedEpp property
    #[zbus(property)]
    fn throttle_balanced_epp(&self) -> zbus::Result<CPUEPP>;
    #[zbus(property)]
    fn set_throttle_balanced_epp(&self, epp: CPUEPP) -> zbus::Result<()>;

    /// ThrottlePerformanceEpp property
    #[zbus(property)]
    fn throttle_performance_epp(&self) -> zbus::Result<CPUEPP>;
    #[zbus(property)]
    fn set_throttle_performance_epp(&self, epp: CPUEPP) -> zbus::Result<()>;

    /// ThrottlePolicyLinkedEpp property
    #[zbus(property)]
    fn throttle_policy_linked_epp(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_throttle_policy_linked_epp(&self, value: bool) -> zbus::Result<()>;

    /// ThrottlePolicyOnAc property
    #[zbus(property)]
    fn throttle_policy_on_ac(&self) -> zbus::Result<ThrottlePolicy>;
    #[zbus(property)]
    fn set_throttle_policy_on_ac(&self, throttle_policy: ThrottlePolicy) -> zbus::Result<()>;

    /// ChangeThrottlePolicyOnAc property
    #[zbus(property)]
    fn change_throttle_policy_on_ac(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_change_throttle_policy_on_ac(&self, change: bool) -> zbus::Result<()>;

    /// ThrottlePolicyOnBattery property
    #[zbus(property)]
    fn throttle_policy_on_battery(&self) -> zbus::Result<ThrottlePolicy>;
    #[zbus(property)]
    fn set_throttle_policy_on_battery(&self, throttle_policy: ThrottlePolicy) -> zbus::Result<()>;

    /// ChangeThrottlePolicyOnAc property
    #[zbus(property)]
    fn change_throttle_policy_on_battery(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_change_throttle_policy_on_battery(&self, change: bool) -> zbus::Result<()>;

    /// ThrottleQuietEpp property
    #[zbus(property)]
    fn throttle_quiet_epp(&self) -> zbus::Result<CPUEPP>;
    #[zbus(property)]
    fn set_throttle_quiet_epp(&self, epp: CPUEPP) -> zbus::Result<()>;

    /// ThrottlePolicy property
    #[zbus(property)]
    fn throttle_thermal_policy(&self) -> zbus::Result<ThrottlePolicy>;
    #[zbus(property)]
    fn set_throttle_thermal_policy(&self, throttle_policy: ThrottlePolicy) -> zbus::Result<()>;
}
