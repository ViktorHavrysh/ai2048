use search_tree::SearchTree;
use searcher::{ExpectiMaxer, SearchResult, Searcher};
use heuristic::Heuristic;
use board::Board;

pub struct Agent<H: Heuristic> {
    search_tree: SearchTree,
    searcher: ExpectiMaxer<H>,
}

impl<H: Heuristic> Agent<H> {
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

    pub fn make_decision(&self) -> SearchResult {
        self.searcher.search(&self.search_tree)
    }

    pub fn update_state(&mut self, board: Board) {
        self.search_tree.set_root(board);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use board::Board;
    use heuristic::heat_map::HeatMapHeuristic;

    #[test]
    fn can_make_decision() {
        let board = Board::default().add_random_tile().add_random_tile();
        let heuristic = HeatMapHeuristic::new();
        let agent = Agent::new(board, heuristic, 0.01, 3);

        let result = agent.make_decision();

        assert_eq!(result.root_board, board);
        assert!(result.move_evaluations.len() >= 2);
        assert!(result.move_evaluations.len() <= 4);
    }

    #[test]
    fn can_update_state() {
        let board = Board::default().add_random_tile().add_random_tile();
        let heuristic = HeatMapHeuristic::new();
        let mut agent = Agent::new(board, heuristic, 0.01, 3);

        let result = agent.make_decision();
        let best_move = *result.move_evaluations.keys().nth(0).unwrap();

        agent.update_state(board.make_move(best_move));
        let result = agent.make_decision();

        assert!(result.root_board != board);
    }
}
