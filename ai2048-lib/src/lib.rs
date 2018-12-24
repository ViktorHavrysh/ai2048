//! This crate provides an implementation of a 2048 AI.
#![deny(missing_docs)]

pub mod game_logic;
mod heuristic;

#[cfg_attr(feature = "parallel", path = "searcher_parallel.rs")]
pub mod searcher;
