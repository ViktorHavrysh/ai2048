//! This module represents the AI player. You feed it news from the outside, query it, and it will
//! give you its decision for what you should be doing next.

use board::Board;
use heuristic::Heuristic;
use search_tree::SearchTree;
use searcher::{ExpectiMaxer, SearchResult, Searcher};

/// `Agent` aggregates all the other parts of the library in order to achieve its whole point.
pub struct Agent<H: Heuristic> {
    search_tree: SearchTree,
    searcher: ExpectiMaxer<H>,
}

impl<H: Heuristic> Agent<H> {
    /// Creates a new `Agent`. Requires the starting state of the `Board`, the heuristic to use,
    /// the limit on the probability of reaching `Board` state before it stops the search, and the
    /// maximum depth ot the search.
    pub fn new(starting_state: Board,
               heuristic: H,
               min_probability: f64,
               max_search_depth: u8)
               -> Self {
        Agent {
            search_tree: SearchTree::new(starting_state),
            searcher: ExpectiMaxer::new(min_probability, max_search_depth, heuristic),
        }
    }

    /// Thinks very hard on the current state of the game and decides what move to make.
    pub fn make_decision(&self) -> SearchResult {
        self.searcher.search(&self.search_tree)
    }

    /// Listens what actually happens after the move is made.
    pub fn update_state(&mut self, board: Board) {
        self.search_tree.set_root(board);
    }
}

#[cfg(test)]
mod tests {
    use board::Board;
    use heuristic::composite::CompositeHeuristic;
    use super::*;

    #[test]
    fn can_make_decision() {
        let board = Board::default().add_random_tile().add_random_tile();
        let heuristic = CompositeHeuristic::default();
        let agent = Agent::new(board, heuristic, 0.01, 3);

        let result = agent.make_decision();

        assert_eq!(result.root_board, board);
        assert!(result.move_evaluations.len() >= 2);
        assert!(result.move_evaluations.len() <= 4);
    }

    #[test]
    fn can_update_state() {
        let board = Board::default().add_random_tile().add_random_tile();
        let heuristic = CompositeHeuristic::default();
        let mut agent = Agent::new(board, heuristic, 0.01, 3);

        let result = agent.make_decision();
        let best_move = *result.move_evaluations.keys().nth(0).unwrap();

        agent.update_state(board.make_move(best_move));
        let result = agent.make_decision();

        assert!(result.root_board != board);
    }
}
