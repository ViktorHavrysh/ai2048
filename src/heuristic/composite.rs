use search_tree::PlayerNode;
use std::u16;
use integer_magic::{u8x4_to_u16,u16_to_u8x4};
use super::*;

const MIN: f32 = -1_600_000f32;

const USE_CACHE: bool = true;

#[derive(Default)]
pub struct CompositeHeuristic;

impl Heuristic for CompositeHeuristic {
    #[inline]
    fn eval(&self, node: &PlayerNode) -> f32 {
        if USE_CACHE {
            self.eval_with_cache(node)
        } else {
            self.eval_without_cache(node)
        }
    }
}

impl CompositeHeuristic {
    #[inline]
    fn eval_without_cache(&self, node: &PlayerNode) -> f32 {
        if node.get_children_by_move().is_empty() {
            return MIN;
        }

        (super::get_monotonicity(node.get_board()) * 47) as f32 +
        (super::get_empty_cell_count(node.get_board()) * 270) as f32 +
        (super::get_adjacent(node.get_board()) * 700) as f32 +
        super::get_sum(node.get_board()) * 11.0
    }

    #[inline]
    fn eval_with_cache(&self, node: &PlayerNode) -> f32 {
        if node.get_children_by_move().is_empty() {
            return MIN;
        }

        node.get_board()
            .get_grid()
            .iter()
            .chain(node.get_board().transpose().get_grid().iter())
            .map(|&row| eval_row(row))
            .sum()
    }
}

lazy_static! {
    static ref CACHE: [f32; u16::MAX as usize] = {
        let mut cache = [0f32; u16::MAX as usize];
        for (index, mut row) in cache.iter_mut().enumerate() {
            *row = eval_row_nocache(u16_to_u8x4(index as u16));
        }
        cache
    };
}

#[inline]
fn eval_row(row: [u8; 4]) -> f32 {
    CACHE[u8x4_to_u16(row) as usize]
}

#[inline]
fn eval_row_nocache(row: [u8; 4]) -> f32 {
    let monotonicity = super::get_monotonicity_row(row) as f32 * 47.0;
    let empty = super::get_empty_cell_count_row(row) as f32 * 270.0;
    let adjacent = super::get_adjacent_row(row) as f32 * 700.0;
    let sum = super::get_sum_row(row) * 11.0;

    monotonicity + empty + adjacent + sum
}
