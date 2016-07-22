use super::*;
use search_tree::PlayerNode;

const MIN: f64 = -200_000f64;

#[derive(Default)]
pub struct CompositeHeuristic;

impl Heuristic for CompositeHeuristic {
    fn eval(&self, node: &PlayerNode) -> f64 {
        if node.get_children_by_move().is_empty() {
            return MIN;
        }

        super::get_monotonicity(node.get_board()) // * 47.0 +
        // super::get_empty_cell_count(node.get_board()) * 270.0 +
        // super::get_adjacent_evaluation(node.get_board()) * 700.0 +
        // super::get_sum(node.get_board()) * 11.0
    }
}
