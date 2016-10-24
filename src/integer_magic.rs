#[inline]
pub fn u8x4_to_u16(row: [u8; 4]) -> u16 {
    (((row[0] & 0b1111) as u16) << 12) +
    (((row[1] & 0b1111) as u16) << 8) +
    (((row[2] & 0b1111) as u16) << 4) +
    (((row[3] & 0b1111) as u16))
}

#[inline]
pub fn u16_to_u8x4(row: u16) -> [u8; 4] {
    let row0 = ((row & 0b1111_0000_0000_0000) >> 12) as u8;
    let row1 = ((row & 0b0000_1111_0000_0000) >> 8) as u8;
    let row2 = ((row & 0b0000_0000_1111_0000) >> 4) as u8;
    let row3 = ((row & 0b0000_0000_0000_1111)) as u8;

    [row0, row1, row2, row3]
}