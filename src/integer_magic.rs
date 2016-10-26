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

#[inline]
pub fn u16_to_u8x4(row: u16) -> [u8; 4] {
    let row0 = ((row & 0b1111_0000_0000_0000) >> 12) as u8;
    let row1 = ((row & 0b0000_1111_0000_0000) >> 8) as u8;
    let row2 = ((row & 0b0000_0000_1111_0000) >> 4) as u8;
    let row3 = ((row & 0b0000_0000_0000_1111)) as u8;

    [row0, row1, row2, row3]
}
