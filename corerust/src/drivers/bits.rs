
// least significant --> most significant
pub fn u32_to_u16(value: u32) -> (u16, u16) {
    (value as u16, (value >> 16) as u16)
}

// least significant --> most significant
pub fn u32_to_u8(value: u32) -> (u8, u8, u8, u8) {
    (value as u8, (value >> 8) as u8, (value >> 16) as u8, (value >> 24) as u8)
}

// least significant --> most significant
pub fn u16_to_u8(value: u16) -> (u8, u8) {
    (value as u8, (value >> 8) as u8)
}
