//! Searcher looks for the best move given a game position

use crate::game_logic::{Grid, Move};
use crate::heuristic;
use cfg_if::cfg_if;
use std::collections::HashMap;
use std::f32;
use std::ops::Add;

cfg_if! {
    if #[cfg(feature = "fnv")] {
        type BuildHasher = fnv::FnvBuildHasher;
    } else if #[cfg(feature = "fxhash")] {
        type BuildHasher = fxhash::FxBuildHasher;
    } else if #[cfg(feature = "t1ha")] {
        type BuildHasher = t1ha::T1haBuildHasher;
    } else {
        type BuildHasher = std::collections::hash_map::RandomState;
    }
}

cfg_if! {
    if #[cfg(feature = "hashbrown")] {
        type Cache<K, V> = hashbrown::HashMap<K, V, BuildHasher>;
    } else if #[cfg(feature = "indexmap")] {
        type Cache<K, V> = indexmap::map::IndexMap<K, V, BuildHasher>;
    } else {
        type Cache<K, V> = std::collections::HashMap<K, V, BuildHasher>;
    }
}

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
    /// Search depth
    pub depth: u8,
}

/// Some search statistics
#[derive(Clone, Debug, Default)]
pub struct SearchStats {
    /// Total nodes travelled
    pub nodes: u32,
    /// Final cache size
    pub cache_size: u32,
    /// Evaluated from cache
    pub cache_hits: u32,
    /// Evaluated with heuristic
    pub evals: u32,
    /// Evaluated as average of children
    pub average: u32,
}

impl Add for SearchStats {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SearchStats {
            nodes: self.nodes + other.nodes,
            cache_size: self.cache_size + other.cache_size,
            cache_hits: self.cache_hits + other.cache_hits,
            evals: self.evals + other.evals,
            average: self.average + other.average,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct SearchState {
    cache: Cache<Grid, (f32, f32)>,
    stats: SearchStats,
    min_probability: f32,
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
        search_inner(grid, self.min_probability, self.max_depth)
    }
}

#[cfg(not(feature = "parallel"))]
fn search_inner(grid: Grid, min_probability: f32, max_depth: u8) -> SearchResult {
    let depth = std::cmp::min(
        max_depth as i8,
        std::cmp::max(3, (grid.count_distinct_tiles() as i8) - 2),
    );
    let mut state = SearchState {
        min_probability,
        ..SearchState::default()
    };
    let mut move_evaluations = grid
        .player_moves()
        .map(|(m, b)| {
            let eval = computer_move_eval(b, 1.0f32, depth, &mut state);
            (m, eval)
        })
        .collect::<Vec<_>>();

    move_evaluations.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("Failed to sort evaluations"));

    let best_move = move_evaluations.iter().cloned().next().map(|(mv, _)| mv);

    let move_evaluations = move_evaluations.into_iter().collect();

    state.stats.cache_size = state.cache.len() as u32;

    SearchResult {
        stats: state.stats,
        root_grid: grid,
        move_evaluations,
        best_move,
        depth: depth as u8,
    }
}

#[cfg(feature = "parallel")]
fn search_inner(grid: Grid, min_probability: f32, max_depth: u8) -> SearchResult {
    use rayon::prelude::*;

    let depth = std::cmp::min(
        max_depth as i8,
        std::cmp::max(3, (grid.count_distinct_tiles() as i8) - 2),
    );

    let mut move_evaluations = grid
        .player_moves()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(m, b)| {
            let mut state = SearchState {
                min_probability,
                ..SearchState::default()
            };
            let eval = computer_move_eval(*b, 1.0f32, depth, &mut state);
            state.stats.cache_size = state.cache.len() as u32;
            (*m, eval, state.stats)
        })
        .collect::<Vec<_>>();

    let stats = move_evaluations
        .iter()
        .map(|(_, _, stats)| stats.clone())
        .fold(SearchStats::default(), |a, b| a + b);

    move_evaluations.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("Failed to sort evaluations"));

    let best_move = move_evaluations.iter().map(|(mv, _, _)| *mv).next();

    let move_evaluations = move_evaluations
        .into_iter()
        .map(|(mv, eval, _)| (mv, eval))
        .collect();

    SearchResult {
        root_grid: grid,
        depth: depth as u8,
        stats,
        move_evaluations,
        best_move,
    }
}

fn player_move_eval(grid: Grid, probability: f32, depth: i8, state: &mut SearchState) -> f32 {
    state.stats.nodes += 1;
    state.stats.average += 1;

    grid
        .player_moves()
        .map(|(_, b)| computer_move_eval(b, probability, depth, state))
        .fold(0f32, f32::max)
}

fn computer_move_eval(grid: Grid, probability: f32, depth: i8, state: &mut SearchState) -> f32 {
    state.stats.nodes += 1;

    if depth <= 0 || probability < state.min_probability {
        state.stats.evals += 1;
        return heuristic::eval(grid);
    }

    if let Some(&(stored_probability, eval)) = state.cache.get(&grid) {
        if probability <= stored_probability {
            state.stats.cache_hits += 1;
            return eval;
        }
    }

    state.stats.average += 1;

    let count = grid.count_empty() as f32;

    let prob2 = probability * PROBABILITY_OF2 / count;
    let prob4 = probability * PROBABILITY_OF4 / count;

    let sum_with2 = grid
        .ai_moves_with2()
        .map(|b| player_move_eval(b, prob2, depth - 1, state))
        .sum::<f32>();
    let avg_with2 = sum_with2 / count;

    let sum_with4 = grid
        .ai_moves_with4()
        .map(|b| player_move_eval(b, prob4, depth - 1, state))
        .sum::<f32>();
    let avg_with4 = sum_with4 / count;

    let eval = avg_with2 * PROBABILITY_OF2 + avg_with4 * PROBABILITY_OF4;

    state.cache.insert(grid, (probability, eval));

    eval
}
