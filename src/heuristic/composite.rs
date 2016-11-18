//! This heuristic works pretty much like the heuristic in
//! [nneonneo's ai written in C++](https://github.com/nneonneo/2048-ai)
//! with some tweaks. I've tried to come up with something more efficient
//! (there is some code in a disabled sibling module that works pretty well),
//! but haven't managed so far. I've decided to try to improve in other ways.
//! I haven't made any benchmarks yet, but I think my usage of transposition
//! tables should considerably speed up the search.

use integer_magic::{u8x4_to_u16, u16_to_u8x4};
use search_tree::PlayerNode;
use std::u16;
use super::*;

const MIN: f32 = -1_600_000f32;

/// A heuristic that uses some other heuristics in tandem. Might be better
/// to rewrite as an aggregate of smaller heuristics.
#[derive(Default)]
pub struct CompositeHeuristic;

impl Heuristic for CompositeHeuristic {
    #[inline]
    fn eval(&self, node: &PlayerNode) -> f32 {
        if node.children().is_empty() {
            return MIN;
        }

        node.board()
            .grid()
            .iter()
            .chain(node.board().transpose().grid().iter())
            .map(eval_row)
            .sum()
    }
}

// Pre-cache heuristic for every possible row with values that can fit a nybble
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
fn eval_row(row: &[u8; 4]) -> f32 {
    match u8x4_to_u16(*row) {
        Some(u) => CACHE[u as usize],
        None => eval_row_nocache(*row),
    }
}

const MONOTONICITY_STRENGTH: f32 = 47.0;
const EMPTY_STRENGTH: f32 = 270.0;
const ADJACENT_STRENGTH: f32 = 700.0;
const SUM_STRENGTH: f32 = 11.0;

#[inline]
fn eval_row_nocache(row: [u8; 4]) -> f32 {
    let monotonicity = super::monotonicity_row(row) as f32 * MONOTONICITY_STRENGTH;
    let empty = super::empty_cell_count_row(row) as f32 * EMPTY_STRENGTH;
    let adjacent = super::adjacent_row(row) as f32 * ADJACENT_STRENGTH;
    let sum = super::sum_row(row) * SUM_STRENGTH;

    monotonicity + empty + adjacent + sum
}
