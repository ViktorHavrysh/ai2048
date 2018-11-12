//! A module for Heuristics. The root module provides a trait for implementing
//! the heuristics, while submodules provide some example implementations.

#![allow(dead_code)]

pub mod composite;

use bytecount;
use crate::board::Board;
use crate::search_tree::PlayerNode;
use std::cmp;
use std::i32;

/// A type that can evaluate a board position and return an `f32`, with
/// higher values meaning better outcome
pub trait Heuristic<T>
where
    T: Copy + Default,
{
    /// Analyzes the game state represented by `PlayerNode` and returns
    /// an evaluation
    fn eval(&self, node: &PlayerNode<T>) -> f32;
}

fn empty_cell_count(board: &Board) -> usize {
    board
        .unpack_u8()
        .iter()
        .flatten()
        .filter(|v| **v == 0)
        .count()
}

fn empty_cell_count_row(row: [u8; 4]) -> usize {
    bytecount::count(&row, 0)
}

fn adjacent(board: &Board) -> u16 {
    board
        .unpack_u8()
        .iter()
        .chain(board.transpose().unpack_u8().iter())
        .map(|&row| adjacent_row(row))
        .fold(0u16, |a, b| a as u16 + b as u16)
}

fn adjacent_row(row: [u8; 4]) -> u8 {
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

fn sum(board: &Board) -> f32 {
    -board
        .unpack_u8()
        .iter()
        .flatten()
        .map(|v| (*v as f32).powf(3.5))
        .sum::<f32>()
}

fn sum_row(row: [u8; 4]) -> f32 {
    -row.iter().map(|v| (*v as f32).powf(3.5)).sum::<f32>()
}

fn monotonicity(board: &Board) -> i32 {
    monotonicity_rows(board) + monotonicity_rows(&board.transpose())
}

fn monotonicity_rows(board: &Board) -> i32 {
    board
        .unpack_u8()
        .iter()
        .map(|&row| monotonicity_row(row))
        .sum()
}

fn monotonicity_row(row: [u8; 4]) -> i32 {
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

fn smoothness(board: &Board) -> i32 {
    let grid = board.unpack_u8();

    let mut smoothness = 0;

    for cell_y in 0..4 {
        for cell_x in 0..4 {
            if let Some(neighbor_x) = ((cell_x + 1)..4).filter(|&x| grid[x][cell_y] != 0).nth(0) {
                smoothness += smoothness
                    - i32::abs(grid[cell_x][cell_y] as i32 - grid[neighbor_x][cell_y] as i32);
            }
            if let Some(neighbor_y) = ((cell_y + 1)..4).filter(|&y| grid[cell_x][y] != 0).nth(0) {
                smoothness += smoothness
                    - i32::abs(grid[cell_x][cell_y] as i32 - grid[cell_x][neighbor_y] as i32);
            }
        }
    }

    smoothness
}
