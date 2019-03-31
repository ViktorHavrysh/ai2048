use crate::build_common::{Column, Row, CACHE_SIZE};

include!(concat!(env!("OUT_DIR"), "/build_generated.rs"));

const CACHE_LEFT: [Row; CACHE_SIZE] = move_left_table();
const CACHE_RIGHT: [Row; CACHE_SIZE] = move_right_table();
const CACHE_UP: [Column; CACHE_SIZE] = move_up_table();
const CACHE_DOWN: [Column; CACHE_SIZE] = move_down_table();
const CACHE_HEUR: [f32; CACHE_SIZE] = heur_table();

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
pub(crate) fn lookup_heur(row: Row) -> f32 {
    // Make sure row.0 is still u16
    let row: u16 = row.0;
    unsafe { *CACHE_HEUR.get_unchecked(row as usize) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_common;

    #[test]
    fn caches_are_correct() {
        for (index, row) in build_common::all_rows() {
            assert_eq!(
                build_common::move_row_left(row),
                CACHE_LEFT[index],
                "index = {}",
                index
            );
            assert_eq!(
                build_common::move_row_right(row),
                CACHE_RIGHT[index],
                "index = {}",
                index
            );
            assert_eq!(
                build_common::move_row_up(row),
                CACHE_UP[index],
                "index = {}",
                index
            );
            assert_eq!(
                build_common::move_row_down(row),
                CACHE_DOWN[index],
                "index = {}",
                index
            );
            assert_eq!(
                build_common::eval_row(row),
                CACHE_HEUR[index],
                "index = {}",
                index
            );
        }
    }
}
