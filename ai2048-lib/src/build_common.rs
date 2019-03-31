// a lot of this is used by build.rs at compile time
#![allow(dead_code)]

use std::{cmp, fmt, u16};

pub(crate) const CACHE_SIZE: usize = u16::MAX as usize + 1;

pub(crate) fn all_rows() -> impl Iterator<Item = (usize, Row)> {
    (0..CACHE_SIZE)
        .map(|index| Row::from_index(index))
        .enumerate()
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Default)]
pub(crate) struct Row(pub(crate) u16);

impl fmt::Debug for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let unpacked = self.unpack();
        write!(
            f,
            "[{:0>4b} {:0>4b} {:0>4b} {:0>4b}]",
            unpacked[0], unpacked[1], unpacked[2], unpacked[3]
        )
    }
}

impl Row {
    // Tries to pack four bytes into four nibbles.
    // If a byte doesn't fit a nibble, returns the index of this byte in `Err`.
    pub(crate) fn pack(row: [u8; 4]) -> Result<Row, usize> {
        let mut result = 0;
        for (index, &tile) in row.iter().enumerate() {
            if tile > 0b1111 {
                return Err(index);
            }
            result <<= 4;
            result += u16::from(tile);
        }
        Ok(Row(result))
    }

    pub(crate) fn from_index(index: usize) -> Row {
        Row(index as u16)
    }

    pub(crate) const fn unpack(self) -> [u8; 4] {
        let row = self.0;
        let tile0 = ((row & 0b1111_0000_0000_0000) >> 12) as u8;
        let tile1 = ((row & 0b0000_1111_0000_0000) >> 8) as u8;
        let tile2 = ((row & 0b0000_0000_1111_0000) >> 4) as u8;
        let tile3 = (row & 0b0000_0000_0000_1111) as u8;
        [tile0, tile1, tile2, tile3]
    }

    pub(crate) const fn reverse(self) -> Self {
        Row((self.0 >> 12)
            | ((self.0 >> 4) & 0b0000_0000_1111_0000)
            | ((self.0 << 4) & 0b0000_1111_0000_0000)
            | (self.0 << 12))
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Default)]
pub(crate) struct Column(pub(crate) u64);

impl Column {
    // 0 2 4 8
    // becomes
    // 0
    // 2
    // 4
    // 8
    pub(crate) fn from_row(row: Row) -> Self {
        const COLUMN_MASK: u64 = 0x000F_000F_000F_000F;
        let col = (u64::from(row.0)
            | u64::from(row.0) << 12
            | u64::from(row.0) << 24
            | u64::from(row.0) << 36)
            & COLUMN_MASK;
        Column(col)
    }
}

// Not much effort spent optimizing this, since it's going to be cached anyway
pub(crate) fn move_row_left(row: Row) -> Row {
    let from_row = row.unpack();
    let to_row = move_row_left_raw(from_row);
    Row::pack(to_row).unwrap()
}

pub(crate) fn move_row_right(row: Row) -> Row {
    move_row_left(row.reverse()).reverse()
}

pub(crate) fn move_row_up(row: Row) -> Column {
    Column::from_row(move_row_left(row))
}

pub(crate) fn move_row_down(row: Row) -> Column {
    Column::from_row(move_row_right(row))
}

// Not much effort spent optimizing this, since it's going to be cached anyway
fn move_row_left_raw(from_row: [u8; 4]) -> [u8; 4] {
    let mut to_row = [0; 4];
    let mut last = 0;
    let mut last_index = 0;

    for &tile in from_row.iter() {
        if tile == 0 {
            continue;
        }

        if last == 0 {
            last = tile;
            continue;
        }

        if tile == last {
            to_row[last_index as usize] = last + 1;
            last = 0;
        } else {
            to_row[last_index as usize] = last;
            last = tile;
        }

        last_index += 1;
    }

    if last != 0 {
        to_row[last_index as usize] = last;
    }

    // If there is a tile which does not fit a nibble, merge into a 32768 instead
    to_row.iter_mut().filter(|i| **i > 15).for_each(|i| *i = 15);
    to_row
}

const NOT_LOST: f32 = 200_000f32;
const MONOTONICITY_STRENGTH: f32 = 47.0;
const EMPTY_STRENGTH: f32 = 270.0;
const ADJACENT_STRENGTH: f32 = 700.0;
const SUM_STRENGTH: f32 = 11.0;

pub(crate) fn eval_row(row: Row) -> f32 {
    let empty = empty_tile_count_row(row) * EMPTY_STRENGTH;
    let monotonicity = monotonicity_row(row) * MONOTONICITY_STRENGTH;
    let adjacent = adjacent_row(row) * ADJACENT_STRENGTH;
    let sum = sum_row(row) * SUM_STRENGTH;
    NOT_LOST + monotonicity + empty + adjacent + sum
}

fn empty_tile_count_row(row: Row) -> f32 {
    row.unpack().iter().filter(|&i| *i == 0).count() as f32
}

fn monotonicity_row(row: Row) -> f32 {
    let row = row.unpack();

    let mut left = 0;
    let mut right = 0;

    for (&current, &next) in row.iter().zip(row.iter().skip(1)) {
        if current > next {
            left += i32::from(current).pow(4) - i32::from(next).pow(4);
        } else if next > current {
            right += i32::from(next).pow(4) - i32::from(current).pow(4);
        }
    }

    -cmp::min(left, right) as f32
}

fn adjacent_row(row: Row) -> f32 {
    let row = row.unpack();

    let mut adjacent_count = 0;
    let mut y = 0;

    while y < 3 {
        if row[y] != 0 && row[y] == row[y + 1] {
            adjacent_count += 1;
            y += 2;
        } else {
            y += 1;
        }
    }

    adjacent_count as f32
}

fn sum_row(row: Row) -> f32 {
    let row = row.unpack();
    -row.iter().map(|&v| f32::from(v).powf(3.5)).sum::<f32>()
}
