//! This crate provides an implementation of a 2048 AI.
#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;

pub mod game_logic;
mod heuristic;
pub mod searcher;
