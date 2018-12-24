//! Searcher looks for the best move given a game position

use crate::cache::Cache;
use crate::game_logic::{Grid, Move};
use crate::heuristic;
use std::f32;

use std::collections::HashMap;

/// Return a number of interesting statistics together with a recommendation for the best move.
#[derive(Clone, Debug, Default)]
pub struct SearchResult {
    /// The game state for which analysis was conducted.
    pub root_grid: Grid,
    /// A map of evaluations. Can be empty if the player has no more moves, that is,
    /// in a game over state.
    pub move_evaluations: HashMap<Move, f32>,
    /// The best move, if one exists. Can be `None` if the player has no available
    /// moves, that is, in a game over state.
    pub best_move: Option<Move>,
    /// Some search statistics
    pub stats: SearchStats,
}

/// Some search statistics
#[derive(Clone, Debug, Default)]
pub struct SearchStats {
    /// The size of the cache
    pub cache_size: usize,
    /// The number of cache hits
    pub cache_hits: usize,
    /// Search depth
    pub depth: u8,
}

#[derive(Clone, Debug, Default)]
struct SearchState {
    cache: Cache<Grid, (f32, f32)>,
    hits: usize,
}

/// Searches for the best move at the current grid state
pub struct Searcher {
    min_probability: f32,
    max_depth: u8,
}

const PROBABILITY_OF2: f32 = 0.9;
const PROBABILITY_OF4: f32 = 0.1;

impl Searcher {
    /// Create a new searcher
    pub fn new(min_probability: f32, max_depth: u8) -> Searcher {
        Searcher {
            min_probability,
            max_depth,
        }
    }

    /// Perform a search for the best move
    pub fn search(&self, grid: Grid) -> SearchResult {
        let mut state = SearchState::default();
        let depth = std::cmp::min(
            self.max_depth as i8,
            std::cmp::max(3, (grid.count_distinct_tiles() as i8) - 2),
        );
        let mut move_evaluations = grid
            .player_moves()
            .map(|(m, b)| {
                let eval = self.computer_move_eval(b, 1.0f32, depth, &mut state);
                (m, eval)
            })
            .collect::<Vec<_>>();

        move_evaluations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let best_move = move_evaluations.iter().cloned().next().map(|(mv, _)| mv);

        let move_evaluations = move_evaluations.into_iter().collect();

        let stats = SearchStats {
            cache_size: state.cache.len(),
            cache_hits: state.hits,
            depth: depth as u8,
        };

        SearchResult {
            root_grid: grid,
            move_evaluations,
            best_move,
            stats,
        }
    }

    fn player_move_eval(
        &self,
        grid: Grid,
        probability: f32,
        depth: i8,
        state: &mut SearchState,
    ) -> f32 {
        if let Some(&(stored_probability, eval)) = state.cache.get(&grid) {
            if probability <= stored_probability {
                state.hits += 1;
                return eval;
            }
        }

        let eval = if grid.game_over() {
            0f32
        } else if depth <= 0 || probability < self.min_probability {
            heuristic::eval(grid)
        } else {
            grid.player_moves()
                .map(|(_, b)| self.computer_move_eval(b, probability, depth, state))
                .fold(f32::NAN, f32::max)
        };

        state.cache.insert(grid, (probability, eval));

        eval
    }

    fn computer_move_eval(
        &self,
        grid: Grid,
        probability: f32,
        depth: i8,
        state: &mut SearchState,
    ) -> f32 {
        let count = grid.count_empty() as f32;

        let prob2 = probability * PROBABILITY_OF2 / count;
        let prob4 = probability * PROBABILITY_OF4 / count;

        let sum_with2 = grid
            .ai_moves_with2()
            .map(|b| self.player_move_eval(b, prob2, depth - 1, state))
            .sum::<f32>();
        let avg_with2 = sum_with2 / count;

        let sum_with4 = grid
            .ai_moves_with4()
            .map(|b| self.player_move_eval(b, prob4, depth - 1, state))
            .sum::<f32>();
        let avg_with4 = sum_with4 / count;

        avg_with2 * PROBABILITY_OF2 + avg_with4 * PROBABILITY_OF4
    }
}
