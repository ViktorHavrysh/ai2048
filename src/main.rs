extern crate ai2048_lib;

use ai2048_lib::SearchResult;
use ai2048_lib::SearchStatistics;
use ai2048_lib::agent::Agent;
use ai2048_lib::board::Board;
use ai2048_lib::board::MOVES;
use ai2048_lib::heuristic::composite::CompositeHeuristic;

use std::fmt::{self, Write};

#[derive(Debug)]
enum Error {
    Fmt(fmt::Error),
}

impl From<fmt::Error> for Error {
    fn from(error: fmt::Error) -> Self {
        Error::Fmt(error)
    }
}

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Error> {
    let heuristic = CompositeHeuristic::default();

    let mut board = Board::default().add_random_tile().add_random_tile();
    let mut agent = Agent::new(board, heuristic, 0.001, 6);
    let mut aggregate_search_statistics = SearchStatistics::default();

    loop {
        let result = agent.make_decision();
        aggregate_search_statistics += result.search_statistics;

        println!("{}", build_display(&result, &aggregate_search_statistics)?);

        if let Some((mv, _)) = result.best_move {
            board = board.make_move(mv).add_random_tile();
            agent.update_state(board);
        } else {
            println!("Game over!");
            return Ok(());
        }
    }
}

fn build_display(
    result: &SearchResult,
    aggregate_stats: &SearchStatistics,
) -> Result<String, fmt::Error> {
    let mut s = String::new();
    write!(&mut s, "{}[2J", 27 as char)?; // clear screen

    writeln!(&mut s, "{}", result.root_board)?;
    writeln!(&mut s, "{}", result.search_statistics)?;
    writeln!(&mut s, "Total:\n{}", aggregate_stats)?;

    for mv in &MOVES {
        write!(&mut s, "{:?}: ", mv)?;
        match result.move_evaluations.get(mv) {
            Some(eval) => writeln!(&mut s, "{}", eval)?,
            None => writeln!(&mut s, "invalid")?,
        }
    }

    if let Some((_, eval)) = result.best_move {
        writeln!(&mut s, "Best: {}", eval)?;
    }

    Ok(s)
}
