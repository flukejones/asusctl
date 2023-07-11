//! Older code that is not useful but stillr elevant as a reference

/// # Bits for newer 0x18c6, 0x19B6, 0x1a30, keyboard models
///
/// | Byte 1 | Byte 2  | Byte 3  | Byte 4  | Label    |
/// |--------|---------|---------|---------|----------|
/// |00000001| 00000000| 00000000| 00000000|boot_logo_|
/// |00000010| 00000000| 00000000| 00000000|boot_keyb_|
/// |00000100| 00000000| 00000000| 00000000|awake_logo|
/// |00001000| 00000000| 00000000| 00000000|awake_keyb|
/// |00010000| 00000000| 00000000| 00000000|sleep_logo|
/// |00100000| 00000000| 00000000| 00000000|sleep_keyb|
/// |01000000| 00000000| 00000000| 00000000|shut_logo_|
/// |10000000| 00000000| 00000000| 00000000|shut_keyb_|
/// |00000000| 00000010| 00000000| 00000000|boot_bar__|
/// |00000000| 00000100| 00000000| 00000000|awake_bar_|
/// |00000000| 00001000| 00000000| 00000000|sleep_bar_|
/// |00000000| 00010000| 00000000| 00000000|shut_bar__|
/// |00000000| 00000000| 00000001| 00000000|boot_lid__|
/// |00000000| 00000000| 00000010| 00000000|awkae_lid_|
/// |00000000| 00000000| 00000100| 00000000|sleep_lid_|
/// |00000000| 00000000| 00001000| 00000000|shut_lid__|
/// |00000000| 00000000| 00000000| 00000001|boot_rear_|
/// |00000000| 00000000| 00000000| 00000010|awake_rear|
/// |00000000| 00000000| 00000000| 00000100|sleep_rear|
/// |00000000| 00000000| 00000000| 00001000|shut_rear_|
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AuraDevRog2 {
    BootLogo = 1,
    BootKeyb = 1 << 1,
    AwakeLogo = 1 << 2,
    AwakeKeyb = 1 << 3,
    SleepLogo = 1 << 4,
    SleepKeyb = 1 << 5,
    ShutdownLogo = 1 << 6,
    ShutdownKeyb = 1 << 7,
    BootBar = 1 << (7 + 2),
    AwakeBar = 1 << (7 + 3),
    SleepBar = 1 << (7 + 4),
    ShutdownBar = 1 << (7 + 5),
    BootLid = 1 << (15 + 1),
    AwakeLid = 1 << (15 + 2),
    SleepLid = 1 << (15 + 3),
    ShutdownLid = 1 << (15 + 4),
    BootRearGlow = 1 << (23 + 1),
    AwakeRearGlow = 1 << (23 + 2),
    SleepRearGlow = 1 << (23 + 3),
    ShutdownRearGlow = 1 << (23 + 4),
}

impl From<AuraDevRog2> for u32 {
    fn from(a: AuraDevRog2) -> Self {
        a as u32
    }
}

impl AuraDevRog2 {
    pub fn to_bytes(control: &[Self]) -> [u8; 4] {
        let mut a: u32 = 0;
        for n in control {
            a |= *n as u32;
        }
        [
            (a & 0xff) as u8,
            ((a & 0xff00) >> 8) as u8,
            ((a & 0xff0000) >> 16) as u8,
            ((a & 0xff000000) >> 24) as u8,
        ]
    }

    pub const fn dev_id() -> &'static str {
        "0x196b"
    }
}

#[cfg(test)]
mod tests {
    use crate::deprecated::AuraDevRog2;

    #[test]
    fn check_0x19b6_control_bytes_binary_rep() {
        fn to_binary_string(bytes: &[AuraDevRog2]) -> String {
            let bytes = AuraDevRog2::to_bytes(bytes);
            format!(
                "{:08b}, {:08b}, {:08b}, {:08b}",
                bytes[0], bytes[1], bytes[2], bytes[3]
            )
        }

        let boot_logo_ = to_binary_string(&[AuraDevRog2::BootLogo]);
        let boot_keyb_ = to_binary_string(&[AuraDevRog2::BootKeyb]);
        let sleep_logo = to_binary_string(&[AuraDevRog2::SleepLogo]);
        let sleep_keyb = to_binary_string(&[AuraDevRog2::SleepKeyb]);
        let awake_logo = to_binary_string(&[AuraDevRog2::AwakeLogo]);
        let awake_keyb = to_binary_string(&[AuraDevRog2::AwakeKeyb]);
        let shut_logo_ = to_binary_string(&[AuraDevRog2::ShutdownLogo]);
        let shut_keyb_ = to_binary_string(&[AuraDevRog2::ShutdownKeyb]);
        let boot_bar__ = to_binary_string(&[AuraDevRog2::BootBar]);
        let awake_bar_ = to_binary_string(&[AuraDevRog2::AwakeBar]);
        let sleep_bar_ = to_binary_string(&[AuraDevRog2::SleepBar]);
        let shut_bar__ = to_binary_string(&[AuraDevRog2::ShutdownBar]);
        let boot_lid__ = to_binary_string(&[AuraDevRog2::BootLid]);
        let awkae_lid_ = to_binary_string(&[AuraDevRog2::AwakeLid]);
        let sleep_lid_ = to_binary_string(&[AuraDevRog2::SleepLid]);
        let shut_lid__ = to_binary_string(&[AuraDevRog2::ShutdownLid]);
        let boot_rear_ = to_binary_string(&[AuraDevRog2::BootRearGlow]);
        let awake_rear = to_binary_string(&[AuraDevRog2::AwakeRearGlow]);
        let sleep_rear = to_binary_string(&[AuraDevRog2::SleepRearGlow]);
        let shut_rear_ = to_binary_string(&[AuraDevRog2::ShutdownRearGlow]);

        assert_eq!(boot_logo_, "00000001, 00000000, 00000000, 00000000");
        assert_eq!(boot_keyb_, "00000010, 00000000, 00000000, 00000000");
        assert_eq!(awake_logo, "00000100, 00000000, 00000000, 00000000");
        assert_eq!(awake_keyb, "00001000, 00000000, 00000000, 00000000");
        assert_eq!(sleep_logo, "00010000, 00000000, 00000000, 00000000");
        assert_eq!(sleep_keyb, "00100000, 00000000, 00000000, 00000000");
        assert_eq!(shut_logo_, "01000000, 00000000, 00000000, 00000000");
        assert_eq!(shut_keyb_, "10000000, 00000000, 00000000, 00000000");
        //
        assert_eq!(boot_bar__, "00000000, 00000010, 00000000, 00000000");
        assert_eq!(awake_bar_, "00000000, 00000100, 00000000, 00000000");
        assert_eq!(sleep_bar_, "00000000, 00001000, 00000000, 00000000");
        assert_eq!(shut_bar__, "00000000, 00010000, 00000000, 00000000");
        //
        assert_eq!(boot_lid__, "00000000, 00000000, 00000001, 00000000");
        assert_eq!(awkae_lid_, "00000000, 00000000, 00000010, 00000000");
        assert_eq!(sleep_lid_, "00000000, 00000000, 00000100, 00000000");
        assert_eq!(shut_lid__, "00000000, 00000000, 00001000, 00000000");
        //
        assert_eq!(boot_rear_, "00000000, 00000000, 00000000, 00000001");
        assert_eq!(awake_rear, "00000000, 00000000, 00000000, 00000010");
        assert_eq!(sleep_rear, "00000000, 00000000, 00000000, 00000100");
        assert_eq!(shut_rear_, "00000000, 00000000, 00000000, 00001000");

        // All on
        let byte1 = [
            AuraDevRog2::BootLogo,
            AuraDevRog2::BootKeyb,
            AuraDevRog2::SleepLogo,
            AuraDevRog2::SleepKeyb,
            AuraDevRog2::AwakeLogo,
            AuraDevRog2::AwakeKeyb,
            AuraDevRog2::ShutdownLogo,
            AuraDevRog2::ShutdownKeyb,
            AuraDevRog2::BootBar,
            AuraDevRog2::AwakeBar,
            AuraDevRog2::SleepBar,
            AuraDevRog2::ShutdownBar,
            AuraDevRog2::AwakeLid,
            AuraDevRog2::BootLid,
            AuraDevRog2::SleepLid,
            AuraDevRog2::ShutdownLid,
            AuraDevRog2::AwakeRearGlow,
            AuraDevRog2::BootRearGlow,
            AuraDevRog2::SleepRearGlow,
            AuraDevRog2::ShutdownRearGlow,
        ];
        let out = to_binary_string(&byte1);
        assert_eq!(out, "11111111, 00011110, 00001111, 00001111");
    }
}
