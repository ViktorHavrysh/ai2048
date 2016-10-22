
use fnv::FnvHashMap;
use search_tree::PlayerNode;
use std::cell::RefCell;
use super::*;

const MIN: f64 = -1_600_000f64;

const USE_CACHE: bool = false;

#[derive(Default)]
pub struct CompositeHeuristic {
    cache: RefCell<FnvHashMap<[u8; 4], f64>>,
}

impl Heuristic for CompositeHeuristic {
    #[inline]
    fn eval(&self, node: &PlayerNode) -> f64 {
        if USE_CACHE {
            self.eval_with_cache(node)
        } else {
            self.eval_without_cache(node)
        }
    }
}

impl CompositeHeuristic {
    #[inline]
    fn eval_without_cache(&self, node: &PlayerNode) -> f64 {
        if node.get_children_by_move().is_empty() {
            return MIN;
        }

        (super::get_monotonicity(node.get_board()) * 47) as f64 +
        super::get_empty_cell_count(node.get_board()) * 270.0 +
        super::get_adjacent_evaluation(node.get_board()) * 700.0 +
        super::get_sum(node.get_board()) * 11.0
    }

    #[inline]
    fn eval_with_cache(&self, node: &PlayerNode) -> f64 {
        if node.get_children_by_move().is_empty() {
            return MIN;
        }

        let mut cache = self.cache.borrow_mut();

        node.get_board()
            .get_grid()
            .iter()
            .chain(node.get_board().transpose().get_grid().iter())
            .map(|&row| eval_row(row, &mut cache))
            .fold(0f64, |a, b| a + b)
    }
}

#[inline]
fn eval_row(row: [u8; 4], cache: &mut FnvHashMap<[u8; 4], f64>) -> f64 {
    *cache.entry(row).or_insert_with(|| eval_row_nocache(row))
}

#[inline]
fn eval_row_nocache(row: [u8; 4]) -> f64 {
    let monotonicity = (super::get_monotonicity_row(row) * 47) as f64;
    let empty = (super::get_empty_cell_count_row(row) as i32 * 270) as f64;
    let adjacent = (super::get_adjacent_row(row) as i32 * 700) as f64;
    let sum = super::get_sum_row(row) * 11.0;

    monotonicity + empty + adjacent + sum
}
