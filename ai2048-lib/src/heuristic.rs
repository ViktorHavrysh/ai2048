use crate::game_logic::{Grid, Row};
use std::{cmp, i32, u16};

pub(crate) fn eval(grid: Grid) -> f32 {
    grid.rows()
        .iter()
        .chain(grid.transpose().rows().iter())
        .map(|&r| eval_row(r))
        .sum()
}

fn eval_row(row: Row) -> f32 {
    unsafe { *CACHE.get_unchecked(row.0 as usize) }
}

// Pre-cache heuristic for every possible row with values that can fit a nibble
lazy_static! {
    static ref CACHE: Box<[f32]> = {
        let mut vec = vec![0f32; u16::MAX as usize];
        for (index, row) in vec.iter_mut().enumerate() {
            *row = eval_row_nocache(Row(index as u16));
        }
        vec.into()
    };
}

const NOT_LOST: f32 = 1_600_000f32;
const MONOTONICITY_STRENGTH: f32 = 47.0;
const EMPTY_STRENGTH: f32 = 270.0;
const ADJACENT_STRENGTH: f32 = 700.0;
const SUM_STRENGTH: f32 = 11.0;

fn eval_row_nocache(row: Row) -> f32 {
    let row = row.unpack();

    let empty = empty_tile_count_row(row) * EMPTY_STRENGTH;
    let monotonicity = monotonicity_row(row) * MONOTONICITY_STRENGTH;
    let adjacent = adjacent_row(row) * ADJACENT_STRENGTH;
    let sum = sum_row(row) * SUM_STRENGTH;

    NOT_LOST + monotonicity + empty + adjacent + sum
}

fn empty_tile_count_row(row: [u8; 4]) -> f32 {
    bytecount::count(&row, 0) as f32
}

fn monotonicity_row(row: [u8; 4]) -> f32 {
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

fn adjacent_row(row: [u8; 4]) -> f32 {
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

fn sum_row(row: [u8; 4]) -> f32 {
    -row.iter().map(|v| f32::from(*v).powf(3.5)).sum::<f32>()
}
