use crate::build_common::{CACHE_SIZE, Row, Column};

include!(concat!(env!("OUT_DIR"), "/build_generated.rs"));

const CACHE_LEFT: [Row; CACHE_SIZE] = move_left_table();
const CACHE_RIGHT: [Row; CACHE_SIZE] = move_right_table();
const CACHE_UP: [Column; CACHE_SIZE] = move_up_table();
const CACHE_DOWN: [Column; CACHE_SIZE] = move_down_table();

// Safety: these are safe because caches are populated for every possible u16 value
pub(crate) fn lookup_left(row: Row) -> Row {
    // Make sure row.0 is still u16
    let row: u16 = row.0;
    unsafe { *CACHE_LEFT.get_unchecked(row as usize) }
}
pub(crate) fn lookup_right(row: Row) -> Row {
    // Make sure row.0 is still u16
    let row: u16 = row.0;
    unsafe { *CACHE_RIGHT.get_unchecked(row as usize) }
}
pub(crate) fn lookup_up(row: Row) -> Column {
    // Make sure row.0 is still u16
    let row: u16 = row.0;
    unsafe { *CACHE_UP.get_unchecked(row as usize) }
}
pub(crate) fn lookup_down(row: Row) -> Column {
    // Make sure row.0 is still u16
    let row: u16 = row.0;
    unsafe { *CACHE_DOWN.get_unchecked(row as usize) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_common::{self, CACHE_SIZE, Row};

    #[test]
    fn caches_are_correct() {
        for index in 0..CACHE_SIZE {
            assert_eq!(build_common::move_row_left(Row::from_index(index)), CACHE_LEFT[index], "index = {}", index);
            assert_eq!(build_common::move_row_right(Row::from_index(index)), CACHE_RIGHT[index], "index = {}", index);
            assert_eq!(build_common::move_row_up(Row::from_index(index)), CACHE_UP[index], "index = {}", index);
            assert_eq!(build_common::move_row_down(Row::from_index(index)), CACHE_DOWN[index], "index = {}", index);
        }
    }
}