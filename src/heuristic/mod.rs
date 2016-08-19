#![allow(dead_code)]

pub mod composite;

use std::i32;
use std::cmp;

use search_tree::PlayerNode;
use board::Board;

use itertools::Itertools;

pub trait Heuristic {
    fn eval(&self, &PlayerNode) -> f64;
}

fn get_empty_cell_count(board: &Board) -> f64 {
    board.get_grid().iter().flatten().filter(|&&v| v == 0).count() as f64
}

fn get_empty_cell_count_row(row: [u8; 4]) -> usize {
    row.iter().filter(|&&v| v == 0).count()
}

fn get_adjacent_evaluation(board: &Board) -> f64 {
    board.get_grid()
        .iter()
        .chain(board.transpose().get_grid().iter())
        .map(|&row| get_adjacent_row(row))
        .fold(0u8, |a, b| a + b) as f64
}

#[inline]
fn get_adjacent_row(row: [u8; 4]) -> u8 {
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

    adjacent_count
}

#[inline]
fn get_sum(board: &Board) -> f64 {
    -board.get_grid().iter().flatten().map(|&v| (v as f64).powf(3.5)).fold(0f64, |a, b| a + b)
}

fn get_sum_row(row: [u8; 4]) -> f64 {
    -row.iter().map(|&v| (v as f64).powf(3.5)).fold(0f64, |a, b| a + b)
}

fn get_monotonicity_rows(board: &Board) -> i32 {
    let mut total = 0;

    for &row in board.get_grid() {
        total += get_monotonicity_row(row);
    }

    total
}

fn get_monotonicity_row(row: [u8; 4]) -> i32 {
    let mut left = 0;
    let mut right = 0;

    for (&current, &next) in row.iter().zip(row.iter().skip(1)) {
        if current > next {
            left += (current as i32).pow(4) - (next as i32).pow(4);
        } else if next > current {
            right += (next as i32).pow(4) - (current as i32).pow(4);
        }
    }

    -cmp::min(left, right)
}

fn get_monotonicity(board: &Board) -> i32 {
    get_monotonicity_rows(board) + get_monotonicity_rows(&board.transpose())
}

fn get_smoothness(board: &Board) -> f64 {
    let grid = board.get_grid();

    let mut smoothness = 0;

    for cell_y in 0..4 {
        for cell_x in 0..4 {
            if let Some(neighbor_x) = ((cell_x + 1)..4).filter(|&x| grid[x][cell_y] != 0).nth(0) {
                smoothness += smoothness -
                              i32::abs(grid[cell_x][cell_y] as i32 -
                                       grid[neighbor_x][cell_y] as i32);
            }
            if let Some(neighbor_y) = ((cell_y + 1)..4).filter(|&y| grid[cell_x][y] != 0).nth(0) {
                smoothness += smoothness -
                              i32::abs(grid[cell_x][cell_y] as i32 -
                                       grid[cell_x][neighbor_y] as i32);
            }
        }
    }

    smoothness as f64
}
