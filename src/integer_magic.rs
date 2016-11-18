/// Compress an array of 4 bytes into a single 2-byte integer if every byte
/// can fit a nybble by discarding the most significant nybble of every byte.
/// Returns `None` if one of the values can't fit a nybble.
#[inline]
pub fn u8x4_to_u16(row: [u8; 4]) -> Option<u16> {
    let mut result = 0;
    for &cell in &row {
        if cell > 0b1111 {
            return None;
        }

        result <<= 4;
        result += cell as u16;
    }

    Some(result)
}

/// Unpacks a 2-byte integer into an array of 4 bytes by splitting the the integer
/// into 4 nybbles.
#[inline]
pub fn u16_to_u8x4(row: u16) -> [u8; 4] {
    let row0 = ((row & 0b1111_0000_0000_0000) >> 12) as u8;
    let row1 = ((row & 0b0000_1111_0000_0000) >> 8) as u8;
    let row2 = ((row & 0b0000_0000_1111_0000) >> 4) as u8;
    let row3 = ((row & 0b0000_0000_0000_1111)) as u8;

    [row0, row1, row2, row3]
}
