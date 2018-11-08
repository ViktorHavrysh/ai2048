extern crate ai2048_lib;
extern crate futures;
extern crate futures_cpupool;

use ai2048_lib::{AggregateSearchStatistics, SearchResult};
use ai2048_lib::agent::Agent;
use ai2048_lib::board::{Board, MOVES};
use ai2048_lib::heuristic::composite::CompositeHeuristic;
use futures::Future;
use futures_cpupool::CpuPool;
use std::fmt::{self, Write};
use std::sync::mpsc;

const MIN_PROBABILITY: f32 = 0.001;
const SEARCH_DEPTH: u8 = 6;

#[derive(Debug)]
enum Error {
    Fmt(fmt::Error),
    Recv(mpsc::RecvError),
    Send(mpsc::SendError<Signal>),
}

#[derive(Debug)]
enum Signal {
    Stop,
    Display(SearchResult),
}

impl From<fmt::Error> for Error {
    fn from(error: fmt::Error) -> Self {
        Error::Fmt(error)
    }
}

impl From<mpsc::RecvError> for Error {
    fn from(error: mpsc::RecvError) -> Self {
        Error::Recv(error)
    }
}

impl From<mpsc::SendError<Signal>> for Error {
    fn from(error: mpsc::SendError<Signal>) -> Self {
        Error::Send(error)
    }
}

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Error> {
    let pool = CpuPool::new_num_cpus();
    let (tx, rx) = mpsc::channel();

    let display_loop = pool.spawn_fn(move || {
        let mut aggregate_search_statistics = AggregateSearchStatistics::default();
        loop {
            let message = rx.recv()?;

            match message {
                Signal::Stop => break,
                Signal::Display(result) => {
                    aggregate_search_statistics += result.search_statistics.into();
                    println!("{}", build_display(&result, &aggregate_search_statistics)?);
                }
            };
        }
        println!("Game over!");
        Ok(())
    });

    let compute_loop = pool.spawn_fn(move || {
        let heuristic = CompositeHeuristic::default();
        let mut board = Board::default().add_random_tile().add_random_tile();
        let mut agent = Agent::new(board, heuristic, MIN_PROBABILITY, SEARCH_DEPTH);
        loop {
            let result = agent.make_decision();
            tx.send(Signal::Display(result.clone()))?;

            if let Some((mv, _)) = result.best_move {
                board = board.make_move(mv).add_random_tile();
                agent.update_state(board);
            } else {
                tx.send(Signal::Stop)?;
                let res: Result<(), Error> = Ok(());
                return res;
            }
        }
    });

    compute_loop.join(display_loop).wait()?;

    Ok(())
}

fn build_display(
    result: &SearchResult,
    aggregate_stats: &AggregateSearchStatistics,
) -> Result<String, fmt::Error> {
    let mut s = String::new();
    write!(&mut s, "{}[2J", 27 as char)?; // clear screen

    writeln!(&mut s, "{}", result.root_board)?;
    writeln!(&mut s, "{}", result.search_statistics)?;
    writeln!(&mut s, "{}", aggregate_stats)?;

    for mv in &MOVES {
        write!(&mut s, "{:>6}: ", mv)?;
        match result.move_evaluations.get(mv) {
            Some(eval) => writeln!(&mut s, "{eval:>16.*}", 3, eval = eval)?,
            None => writeln!(&mut s, "{:>16}", "illegal move")?,
        }
    }

    if let Some((_, eval)) = result.best_move {
        writeln!(&mut s)?;
        writeln!(&mut s, "  Best: {eval:>16.*}", 3, eval = eval)?;
    }

    writeln!(&mut s)?;

    writeln!(&mut s, "Depth: {}", SEARCH_DEPTH)?;
    writeln!(&mut s, "Cutoff probability: {}", MIN_PROBABILITY)?;

    Ok(s)
}
