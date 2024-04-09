// Only these two packets must be 17 bytes
pub const LED_APPLY: [u8; 17] = [0x5d, 0xb4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const LED_SET: [u8; 17] = [0x5d, 0xb5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

/// Writes out the correct byte string for brightness
pub const fn aura_brightness_bytes(brightness: u8) -> [u8; 17] {
    [
        0x5a, 0xba, 0xc5, 0xc4, brightness, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}
