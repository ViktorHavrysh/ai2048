//! Heuristic to evaluate position

use crate::build_generated::lookup_heur;
use crate::game_logic::Grid;

/// Evaluate grid based on heuristics
pub fn eval(grid: Grid) -> f32 {
    grid.rows()
        .iter()
        .chain(grid.transpose().rows().iter())
        .map(|&r| lookup_heur(r))
        .sum()
}
