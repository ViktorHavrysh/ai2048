//! This is the meat of the library. This module implements an `ExpectiMax` search
//! (`https://en.wikipedia.org/wiki/Expectiminimax_tree` - we don't need a MIN node, since
//! the Computer player is not trying to win). So we're just trying to find the best moves
//! after which, with perfect play, we expect the heuristic value to be the highest, on average.
//!
//! Of course, it's impossible to calculate the whole tree in most board positions, so we stop
//! going deeper into the search tree as soon as we either reach a node whose probability is lower
//! than some value, or as soon as we reach a certain depth, whichever happens first.
//!
//! As soon as that happens, or we reach a terminal (Game Over) node, we run the provided heuristic
//! against the current board state and pass it up.
//!
//! Player nodes are MAX nodes that seek to select the best continuation, and so discard all other
//! evaluations before passing the evaluation up.
//!
//! Computer nodes are AVG nodes that return the weighted average of its child states.

use board::{Board, Move};
use heuristic::Heuristic;
use itertools::Itertools;
use search_tree::{ComputerNode, PlayerNode, SearchTree};
use std::collections::HashMap;
use std::f32;
use std::fmt;
use std::ops::{Add, AddAssign};
use time::{self, Duration};

const PROBABILITY_OF2: f32 = 0.9;
const PROBABILITY_OF4: f32 = 0.1;

const USE_DETAILED_STATS: bool = true;

/// Not sure why I created a trait. I used to experiment a lot with different search methods,
/// but I don't think I'll find a better algorithm than `ExpectiMax` now.
pub trait Searcher<T>
where
    T: Copy + Default,
{
    fn search(&self, search_tree: &SearchTree<T>) -> SearchResult;
}

/// Return a number of interesting statistics together with a recommendation for the best move.
#[derive(Clone, Debug)]
pub struct SearchResult {
    /// Some useful statistics
    pub search_statistics: SearchStatistics,
    /// The game state for which analysis was conducted.
    pub root_board: Board,
    /// A map of evaluations. Can be empty if the player has no more moves, that is,
    /// in a game over state.
    pub move_evaluations: HashMap<Move, f32>,
    /// The best move, if one exists. Can be `None` if the player has no available
    /// moves, that is, in a game over state.
    pub best_move: Option<(Move, f32)>,
}

/// These are the interesting statistics. May add some more later.
#[derive(Clone, Copy, Debug)]
pub struct SearchStatistics {
    /// The time it took for the search to complete.
    pub search_duration: Duration,
    /// The number of search tree nodes visited.
    pub nodes_traversed: usize,
    /// The number of nodes for which the game state was evaluated with a heuristic.
    pub terminal_new: usize,
    /// The number of end nodes for which the heuristic was already cached.
    pub terminal_cached: usize,
    /// The number of shortcuts taken due to reaching the same node at higher level.
    pub shortcuts_hit: usize,
    /// Known unique search tree nodes that represent the Player's turn.
    pub known_player_nodes: usize,
    /// Known unique search tree nodes that represent the Computer's turn.
    pub known_computer_nodes: usize,
    /// New unique game states that the Player can encounter that were found
    /// during this search.
    pub new_player_nodes: usize,
    /// New unique game states that the Computer can encounter that were found
    /// during this search.
    pub new_computer_nodes: usize,
}

/// Statistics aggregated for several searches
#[derive(Clone, Copy, Debug)]
pub struct AggregateSearchStatistics {
    /// Total number of searches
    pub searches: usize,
    /// Total time it took to search
    pub search_duration: Duration,
    /// Total nodes traversed
    pub nodes_traversed: usize,
    /// Total new terminal nodes traversed
    pub terminal_new: usize,
    /// Total cached terminal nodes traversed
    pub terminal_cached: usize,
    /// Total number of shortcuts hit
    pub shortcuts_hit: usize,
    known_player_nodes: usize,
    known_computer_nodes: usize,
    new_player_nodes: usize,
    new_computer_nodes: usize,
}

impl From<SearchStatistics> for AggregateSearchStatistics {
    fn from(ss: SearchStatistics) -> Self {
        AggregateSearchStatistics {
            searches: 1,
            search_duration: ss.search_duration,
            nodes_traversed: ss.nodes_traversed,
            terminal_new: ss.terminal_new,
            terminal_cached: ss.terminal_cached,
            shortcuts_hit: ss.shortcuts_hit,
            known_player_nodes: ss.known_player_nodes,
            known_computer_nodes: ss.known_computer_nodes,
            new_player_nodes: ss.new_player_nodes,
            new_computer_nodes: ss.new_computer_nodes,
        }
    }
}

impl Add for AggregateSearchStatistics {
    type Output = AggregateSearchStatistics;

    fn add(self, other: Self) -> Self::Output {
        AggregateSearchStatistics {
            searches: self.searches + other.searches,
            search_duration: self.search_duration + other.search_duration,
            nodes_traversed: self.nodes_traversed + other.nodes_traversed,
            terminal_new: self.terminal_new + other.terminal_new,
            terminal_cached: self.terminal_cached + other.terminal_cached,
            shortcuts_hit: self.shortcuts_hit + other.shortcuts_hit,
            known_player_nodes: self.known_player_nodes + other.known_player_nodes,
            known_computer_nodes: self.known_computer_nodes + other.known_computer_nodes,
            new_player_nodes: self.new_player_nodes + other.new_player_nodes,
            new_computer_nodes: self.new_computer_nodes + other.new_computer_nodes,
        }
    }
}

impl AddAssign for AggregateSearchStatistics {
    #[allow(assign_op_pattern)]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

// Unfortunately, can't derive, since `Duration` apparently doesn't implement it
// (why?)
impl Default for SearchStatistics {
    fn default() -> Self {
        SearchStatistics {
            search_duration: Duration::zero(),
            nodes_traversed: 0,
            terminal_new: 0,
            terminal_cached: 0,
            shortcuts_hit: 0,
            known_player_nodes: 0,
            known_computer_nodes: 0,
            new_player_nodes: 0,
            new_computer_nodes: 0,
        }
    }
}

impl Default for AggregateSearchStatistics {
    fn default() -> Self {
        AggregateSearchStatistics {
            searches: 0,
            search_duration: Duration::zero(),
            nodes_traversed: 0,
            terminal_new: 0,
            terminal_cached: 0,
            shortcuts_hit: 0,
            known_player_nodes: 0,
            known_computer_nodes: 0,
            new_player_nodes: 0,
            new_computer_nodes: 0,
        }
    }
}

// Helper methods to compute some derivative values.
impl SearchStatistics {
    fn known_nodes(&self) -> usize {
        self.known_player_nodes + self.known_computer_nodes
    }
    fn new_nodes(&self) -> usize {
        self.new_player_nodes + self.new_computer_nodes
    }
    fn nodes_per_second(&self) -> u64 {
        (self.nodes_traversed as f32
            * (1_000_000_000f32 / self.search_duration.num_nanoseconds().unwrap() as f32))
            as u64
    }
    fn new_nodes_per_second(&self) -> u64 {
        (self.new_nodes() as f32
            * (1_000_000_000f32 / self.search_duration.num_nanoseconds().unwrap() as f32))
            as u64
    }
}
impl AggregateSearchStatistics {
    fn known_nodes(&self) -> usize {
        self.known_player_nodes + self.known_computer_nodes
    }
    fn new_nodes(&self) -> usize {
        self.new_player_nodes + self.new_computer_nodes
    }
    fn nodes_per_second(&self) -> u64 {
        (self.nodes_traversed as f32
            * (1_000_000_000f32 / self.search_duration.num_nanoseconds().unwrap() as f32))
            as u64
    }
    fn new_nodes_per_second(&self) -> u64 {
        (self.new_nodes() as f32
            * (1_000_000_000f32 / self.search_duration.num_nanoseconds().unwrap() as f32))
            as u64
    }
    fn average_search_duration(&self) -> Duration {
        self.search_duration / (self.searches as i32)
    }
    fn average_new_nodes(&self) -> usize {
        self.new_nodes() / self.searches
    }
    fn average_known_nodes(&self) -> usize {
        self.known_nodes() / self.searches
    }
    fn average_nodes_traversed(&self) -> usize {
        self.nodes_traversed / self.searches
    }
    fn average_terminal_new(&self) -> usize {
        self.terminal_new / self.searches
    }
    fn average_terminal_cached(&self) -> usize {
        self.terminal_cached / self.searches
    }
    fn average_shortcuts_hit(&self) -> usize {
        self.shortcuts_hit / self.searches
    }
}

impl fmt::Display for SearchStatistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Statistics:")?;
        writeln!(
            f,
            "Search duration:                  {}",
            self.search_duration
        )?;
        writeln!(
            f,
            "Known nodes:                      {}",
            self.known_nodes()
        )?;
        writeln!(f, "New nodes:                        {}", self.new_nodes())?;
        writeln!(
            f,
            "New nodes per second:             {}",
            self.new_nodes_per_second()
        )?;

        if USE_DETAILED_STATS {
            writeln!(
                f,
                "Nodes traversed:                  {}",
                self.nodes_traversed
            )?;
            writeln!(
                f,
                "Nodes per second:                 {}",
                self.nodes_per_second()
            )?;
            writeln!(
                f,
                "Shortcuts hit:                    {}",
                self.shortcuts_hit
            )?;
            writeln!(f, "Terminal nodes (new):             {}", self.terminal_new)?;
            writeln!(
                f,
                "Terminal nodes (cached):          {}",
                self.terminal_cached
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for AggregateSearchStatistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Total statistics:")?;
        writeln!(f, "Searches conducted:               {}", self.searches)?;
        writeln!(
            f,
            "Total search duration:            {}",
            self.search_duration
        )?;
        writeln!(
            f,
            "Average search duration:          {}",
            self.average_search_duration()
        )?;
        writeln!(
            f,
            "Average known nodes:              {}",
            self.average_known_nodes()
        )?;
        writeln!(
            f,
            "Average new nodes:                {}",
            self.average_new_nodes()
        )?;
        writeln!(
            f,
            "New nodes per second:             {}",
            self.new_nodes_per_second()
        )?;

        if USE_DETAILED_STATS {
            writeln!(
                f,
                "Average nodes traversed:          {}",
                self.average_nodes_traversed()
            )?;
            writeln!(
                f,
                "Nodes per second:                 {}",
                self.nodes_per_second()
            )?;
            writeln!(
                f,
                "Average shortcuts hit:            {}",
                self.average_shortcuts_hit()
            )?;
            writeln!(
                f,
                "Average terminal nodes (new):     {}",
                self.average_terminal_new()
            )?;
            writeln!(
                f,
                "Average terminal nodes (cached):  {}",
                self.average_terminal_cached()
            )?;
        }

        Ok(())
    }
}

/// Potentially cached data in the evaluated node.
pub type ExpectiMaxerCache = Option<ExpectiMaxerCachedData>;

/// Cached temporary data
#[derive(Copy, Clone, Default)]
pub struct ExpectiMaxerCachedData {
    /// Cached heuristic for a previously seen node
    cached_heuristic: f32,
    /// The maximum probability of encountering this node
    max_probability: f32,
}

/// The main consumer of computational resources of the program.
pub struct ExpectiMaxer<H>
where
    H: Heuristic<ExpectiMaxerCache>,
{
    min_probability: f32,
    max_search_depth: u8,
    heuristic: H,
}

impl<H> Searcher<ExpectiMaxerCache> for ExpectiMaxer<H>
where
    H: Heuristic<ExpectiMaxerCache>,
{
    /// Do the search.
    fn search(&self, search_tree: &SearchTree<ExpectiMaxerCache>) -> SearchResult {
        let mut statistics = SearchStatistics::default();

        // gather some data before starting the search
        let start = time::now_utc();
        let known_player_nodes_start = search_tree.known_player_node_count();
        let known_computer_nodes_start = search_tree.known_computer_node_count();

        // actual search
        let hashmap = self.init(search_tree, &mut statistics);

        // gather some data after finishing the search
        let finish = time::now_utc();
        let elapsed = finish - start;
        let known_player_nodes_finish = search_tree.known_player_node_count();
        let known_computer_nodes_finish = search_tree.known_computer_node_count();

        // compute some deltas
        statistics.search_duration = elapsed;
        statistics.new_computer_nodes = known_computer_nodes_finish - known_computer_nodes_start;
        statistics.new_player_nodes = known_player_nodes_finish - known_player_nodes_start;
        statistics.known_computer_nodes = known_computer_nodes_finish;
        statistics.known_player_nodes = known_player_nodes_finish;

        // find the best evaluation and move
        let best_move = hashmap
            .iter()
            .sorted_by(|a, b| b.1.partial_cmp(a.1).unwrap())
            .into_iter()
            .map(|(mv, eval)| (*mv, *eval))
            .next();

        SearchResult {
            root_board: *search_tree.root().board(),
            move_evaluations: hashmap,
            search_statistics: statistics,
            best_move,
        }
    }
}

impl<H> ExpectiMaxer<H>
where
    H: Heuristic<ExpectiMaxerCache>,
{
    /// Creates a new `ExpectiMaxer`. Require the heuristic to use, the limit probability
    /// lower than which we'll won't search, and the maximum search depth.
    pub fn new(min_probability: f32, max_search_depth: u8, heuristic: H) -> Self {
        assert_ne!(max_search_depth, 0);
        ExpectiMaxer {
            min_probability,
            max_search_depth,
            heuristic,
        }
    }

    fn init(
        &self,
        search_tree: &SearchTree<ExpectiMaxerCache>,
        mut search_statistics: &mut SearchStatistics,
    ) -> HashMap<Move, f32> {
        if search_tree.root().children().is_empty() {
            // Game over
            return HashMap::new();
        }

        search_tree
            .root()
            .children()
            .iter()
            .map(|(m, n)| {
                let eval =
                    self.computer_node_eval(n, self.max_search_depth, 1f32, &mut search_statistics);
                (m, eval)
            })
            .collect()
    }

    fn player_node_eval(
        &self,
        node: &PlayerNode<ExpectiMaxerCache>,
        depth: u8,
        probability: f32,
        mut statistics: &mut SearchStatistics,
    ) -> f32 {
        if USE_DETAILED_STATS {
            statistics.nodes_traversed += 1;
        }

        let data = node.data.get();

        // Have we seen this node on the same level or deeper in the tree?
        if let Some(data) = data {
            if probability <= data.max_probability {
                if USE_DETAILED_STATS {
                    statistics.shortcuts_hit += 1;
                }
                return data.cached_heuristic;
            }
        }

        if node.children().is_empty() || depth == 0 || probability < self.min_probability {
            match data {
                Some(data) => {
                    if USE_DETAILED_STATS {
                        statistics.terminal_cached += 1;
                    }
                    return data.cached_heuristic;
                }
                None => {
                    if USE_DETAILED_STATS {
                        statistics.terminal_new += 1;
                    }
                    let heur = self.heuristic.eval(node);
                    let new_data = Some(ExpectiMaxerCachedData {
                        cached_heuristic: heur,
                        max_probability: probability,
                    });
                    node.data.set(new_data);
                    return heur;
                }
            };
        }

        let heur = node.children()
            .values()
            .map(|n| self.computer_node_eval(n, depth, probability, &mut statistics))
            .fold(f32::NAN, f32::max);

        match data {
            Some(data) if data.max_probability >= probability => (),
            _ => {
                node.data.set(Some(ExpectiMaxerCachedData {
                    cached_heuristic: heur,
                    max_probability: probability,
                }));
            }
        };

        heur
    }

    fn computer_node_eval(
        &self,
        node: &ComputerNode<ExpectiMaxerCache>,
        depth: u8,
        probability: f32,
        mut statistics: &mut SearchStatistics,
    ) -> f32 {
        if USE_DETAILED_STATS {
            statistics.nodes_traversed += 1;
        }
        let children = node.children();
        let count = children.variants() as f32;

        let child_with2_probability = probability * PROBABILITY_OF2 / count;
        let child_with4_probability = probability * PROBABILITY_OF4 / count;

        let avg_with2 = children
            .with2()
            .map(|n| self.player_node_eval(n, depth - 1, child_with2_probability, &mut statistics))
            .sum::<f32>() / count;

        let avg_with4 = children
            .with4()
            .map(|n| self.player_node_eval(n, depth - 1, child_with4_probability, &mut statistics))
            .sum::<f32>() / count;

        avg_with2 * PROBABILITY_OF2 + avg_with4 * PROBABILITY_OF4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use board::Board;
    use heuristic::composite::CompositeHeuristic;
    use search_tree::SearchTree;

    #[test]
    fn can_search_result() {
        let board = Board::default().add_random_tile();
        let search_tree = SearchTree::new(board);
        let heuristic = CompositeHeuristic::default();
        let searcher = ExpectiMaxer::new(0.01, 3, heuristic);

        let result = searcher.search(&search_tree);

        assert_eq!(result.root_board, board);
        assert!(result.move_evaluations.len() >= 2);
        assert!(result.move_evaluations.len() <= 4);
    }
}
