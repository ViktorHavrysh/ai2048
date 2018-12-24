use crate::game_logic::{Grid, Move};
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
    /// Evaluated as game over
    pub over: u32,
    /// Evaluated as average of children
    pub average: u32,
}
