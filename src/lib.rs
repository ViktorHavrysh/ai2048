extern crate rand;
extern crate time;

pub use searcher::{SearchResult, SearchStatistics};

pub mod board;
pub mod agent;
pub mod heuristic;

mod search_tree;
mod searcher;
