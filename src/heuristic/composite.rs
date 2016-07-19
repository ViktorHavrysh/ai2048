use super::*;
use search_tree::PlayerNode;

const MIN: f64 = -1_000_000f64;

pub struct CompositeHeuristic;

impl CompositeHeuristic {
    pub fn new() -> Self {
        CompositeHeuristic {}
    }
}

impl Heuristic for CompositeHeuristic {
    fn eval(&self, node: &PlayerNode) -> f64 {
        if node.get_children_by_move().len() == 0 {
            return MIN;
        }

        super::get_monotonicity(node.get_board()) * 10.0 +
        super::get_empty_cell_count(node.get_board())
    }
}
