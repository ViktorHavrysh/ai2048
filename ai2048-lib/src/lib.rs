//! This crate provides an implementation of a 2048 AI.
#![deny(missing_docs)]

pub mod game_logic;
pub mod heuristic;
mod searcher_data;

#[cfg_attr(feature = "parallel", path = "searcher_parallel.rs")]
pub mod searcher;
