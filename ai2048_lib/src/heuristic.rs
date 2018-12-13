use crate::game_logic::{Board, Row};
use bytecount;
use std::cmp;
use std::i32;
use std::u16;

#[inline]
pub fn eval(board: Board) -> f32 {
    board
        .rows
        .iter()
        .chain(board.transpose().rows.iter())
        .map(|&r| eval_row(r))
        .sum()
}

const MONOTONICITY_STRENGTH: f32 = 47.0;
const EMPTY_STRENGTH: f32 = 270.0;
const ADJACENT_STRENGTH: f32 = 700.0;
const SUM_STRENGTH: f32 = 11.0;

#[inline]
fn eval_row(row: Row) -> f32 {
    CACHE[row.0 as usize]
}

// Pre-cache heuristic for every possible row with values that can fit a nybble
lazy_static! {
    static ref CACHE: [f32; u16::MAX as usize] = {
        let mut cache = [0f32; u16::MAX as usize];
        for (index, row) in cache.iter_mut().enumerate() {
            *row = eval_row_nocache(Row(index as u16));
        }
        cache
    };
}

fn eval_row_nocache(row: Row) -> f32 {
    let row = row.unpack();

    let empty = empty_cell_count_row(row) * EMPTY_STRENGTH;
    let monotonicity = monotonicity_row(row) * MONOTONICITY_STRENGTH;
    let adjacent = adjacent_row(row) * ADJACENT_STRENGTH;
    let sum = sum_row(row) * SUM_STRENGTH;

    monotonicity + empty + adjacent + sum
}

fn empty_cell_count_row(row: [u8; 4]) -> f32 {
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
