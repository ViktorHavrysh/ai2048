//! This crate provides an implementation of an 2048 AI.
//!
//! The `board` module contains 2048 game logic.
//!
//! The `agent` module contains an AI player.
//!
//! The `heuristic` module contains various heuristics that the AI player can use to evaluate
//! board positions and try to maximize. It also contains the `Heuristic` trait that can be used
//! to implement your own heuristic.
//!
//! The `SearchResult` and `SearchStatistics` types are containers for the results of the AI
//! player's evaluation of a position and some interesting statistics.
#[macro_use]
extern crate lazy_static;

pub mod game_logic;
mod heuristic;
pub mod searcher;
