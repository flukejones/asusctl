pub mod zbus;

pub mod controller;

static FAN_TYPE_1_PATH: &str = "/sys/devices/platform/asus-nb-wmi/throttle_thermal_policy";
static FAN_TYPE_2_PATH: &str = "/sys/devices/platform/asus-nb-wmi/fan_boost_mode";
static AMD_BOOST_PATH: &str = "/sys/devices/system/cpu/cpufreq/boost";
