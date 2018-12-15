//! New implementation of searcher, not using SearchTree

use crate::game_logic::{Board, Move};
use crate::heuristic;
use std::f32;

use std::collections::HashMap;
type Cache<K, V> = hashbrown::HashMap<K, V, fnv::FnvBuildHasher>;

/// Return a number of interesting statistics together with a recommendation for the best move.
#[derive(Clone, Debug)]
pub struct SearchResult {
    /// The game state for which analysis was conducted.
    pub root_board: Board,
    /// A map of evaluations. Can be empty if the player has no more moves, that is,
    /// in a game over state.
    pub move_evaluations: HashMap<Move, f32>,
    /// The best move, if one exists. Can be `None` if the player has no available
    /// moves, that is, in a game over state.
    pub best_move: Option<(Move, f32)>,
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
    stats: SearchStats,
    cache: Cache<Board, (f32, f32)>,
    hits: usize,
}

/// Searches for the best move at the current board state
pub struct Searcher {
    min_probability: f32,
}

const PROBABILITY_OF2: f32 = 0.9;
const PROBABILITY_OF4: f32 = 0.1;

impl Searcher {
    /// Create a new searcher
    pub fn new(min_probability: f32) -> Searcher {
        Searcher { min_probability }
    }

    pub fn search(&self, board: Board) -> SearchResult {
        let mut state = SearchState::default();
        let depth = std::cmp::max(3, (board.count_distinct_cells() as i8) - 2);
        let mut move_evaluations = board
            .player_moves()
            .map(|(m, b)| {
                let eval = self.computer_move_eval(b, 1.0f32, depth, &mut state);
                (m, eval)
            })
            .collect::<Vec<_>>();

        move_evaluations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let best_move = move_evaluations.iter().cloned().next();

        let move_evaluations = move_evaluations.into_iter().collect();

        let stats = SearchStats {
            cache_size: state.cache.len(),
            cache_hits: state.hits,
            depth: depth as u8,
        };

        SearchResult {
            root_board: board,
            move_evaluations,
            best_move,
            stats,
        }
    }

    fn player_move_eval(
        &self,
        board: Board,
        probability: f32,
        depth: i8,
        state: &mut SearchState,
    ) -> f32 {
        if let Some(&(stored_probability, eval)) = state.cache.get(&board) {
            if probability <= stored_probability {
                state.hits += 1;
                return eval;
            }
        }

        let eval = if board.game_over() {
            0f32
        } else if depth <= 0 || probability < self.min_probability {
            heuristic::eval(board)
        } else {
            board
                .player_moves()
                .map(|(_, b)| self.computer_move_eval(b, probability, depth, state))
                .fold(f32::NAN, f32::max)
        };

        state.cache.insert(board, (probability, eval));

        eval
    }

    fn computer_move_eval(
        &self,
        board: Board,
        probability: f32,
        depth: i8,
        state: &mut SearchState,
    ) -> f32 {
        let count = board.count_empty() as f32;

        let prob2 = probability * PROBABILITY_OF2 / count;
        let prob4 = probability * PROBABILITY_OF4 / count;

        let sum_with2 = board
            .ai_moves_with2()
            .map(|b| self.player_move_eval(b, prob2, depth - 1, state))
            .sum::<f32>();
        let avg_with2 = sum_with2 / count;

        let sum_with4 = board
            .ai_moves_with4()
            .map(|b| self.player_move_eval(b, prob4, depth - 2, state))
            .sum::<f32>();
        let avg_with4 = sum_with4 / count;

        avg_with2 * PROBABILITY_OF2 + avg_with4 * PROBABILITY_OF4
    }
}
