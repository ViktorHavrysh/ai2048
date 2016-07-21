#![allow(dead_code)]

pub mod composite;

use std::{cmp, i32};
use search_tree::PlayerNode;
use board::Board;

pub trait Heuristic {
    fn eval(&self, &PlayerNode) -> f64;
}

fn get_empty_cell_count(board: &Board) -> f64 {
    board.flatten().iter().filter(|&&v| v == 0).count() as f64
}

fn get_adjacent_evaluation(board: &Board) -> f64 {
    let mut adjacent_count = 0;

    let grid = board.get_grid();

    for y in 0..4 {
        let mut x = 0;
        while x < 3 {
            if grid[x][y] != 0 && grid[x][y] == grid[x + 1][y] {
                adjacent_count += 1;
                x += 2;
            } else {
                x += 1;
            }
        }
    }

    for x in 0..4 {
        let mut y = 0;
        while y < 3 {
            if grid[x][y] != 0 && grid[x][y] == grid[x][y + 1] {
                adjacent_count += 1;
                y += 2;
            } else {
                y += 1;
            }
        }
    }

    adjacent_count as f64
}

fn get_monotonicity(board: &Board) -> f64 {
    let grid = board.get_grid();

    let mut up: isize = 0;
    let mut down: isize = 0;

    for x in 0..4 {
        let mut current = 0;
        let mut next = 1;

        while next < 4 {
            while next < 4 && grid[x][next] == 0 {
                next += 1;
            }
            if next >= 4 {
                next -= 1;
            }

            let current_value = grid[x][current] as isize;
            let next_value = grid[x][next] as isize;

            if current_value > next_value {
                down += next_value - current_value;
            } else if next_value > current_value {
                up += current_value - next_value;
            }

            current = next;
            next += 1;
        }
    }

    let mut right: isize = 0;
    let mut left: isize = 0;

    for y in 0..4 {
        let mut current = 0;
        let mut next = 1;

        while next < 4 {
            while next < 4 && grid[next][y] == 0 {
                next += 1;
            }
            if next >= 4 {
                next -= 1;
            }

            let current_value = grid[current][y] as isize;
            let next_value = grid[next][y] as isize;

            if current_value > next_value {
                right += next_value - current_value;
            } else if next_value > current_value {
                left += current_value - next_value;
            }

            current = next;
            next += 1;
        }
    }

    let result = cmp::max(up, down) + cmp::max(left, right);

    result as f64
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
