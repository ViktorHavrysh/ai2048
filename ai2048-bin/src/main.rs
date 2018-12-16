use ai2048_lib::game_logic::{Grid, MOVES};
use ai2048_lib::searcher::{SearchResult, Searcher};
use chrono::prelude::*;
use futures::Future;
use futures_cpupool::CpuPool;
use std::alloc::System;
use std::fmt::{self, Write};
use std::sync::mpsc;

const MIN_PROBABILITY: f32 = 0.0001;

#[derive(Debug)]
enum Error {
    Fmt(fmt::Error),
    Recv(mpsc::RecvError),
    Send(mpsc::SendError<Signal>),
}

#[derive(Debug)]
enum Signal {
    Stop,
    Display(SearchResult, i32, chrono::Duration, chrono::Duration),
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

#[global_allocator]
static A: System = System;

fn main() -> Result<(), Error> {
    let pool = CpuPool::new_num_cpus();
    let (tx, rx) = mpsc::channel();

    let display_loop = pool.spawn_fn(move || {
        loop {
            let message = rx.recv()?;

            match message {
                Signal::Stop => break,
                Signal::Display(result, moves, one, overall) => {
                    println!("{}", build_display(&result, moves, one, overall)?);
                }
            };
        }
        println!("Game over!");
        Ok(())
    });

    let compute_loop = pool.spawn_fn(move || {
        let searcher = Searcher::new(MIN_PROBABILITY);
        let mut grid = Grid::default().add_random_tile().add_random_tile();
        let start_overall = Utc::now();
        let mut moves = 0;
        loop {
            moves += 1;
            let start_one = Utc::now();
            let result = searcher.search(grid);
            let end = Utc::now();
            tx.send(Signal::Display(
                result.clone(),
                moves,
                end - start_one,
                end - start_overall,
            ))?;

            if let Some((mv, _)) = result.best_move {
                grid = grid.make_move(mv).add_random_tile();
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
    moves: i32,
    one: chrono::Duration,
    overall: chrono::Duration,
) -> Result<String, fmt::Error> {
    let mut s = String::new();
    write!(&mut s, "{}[2J", 27 as char)?; // clear screen

    writeln!(&mut s, "{}", result.root_grid)?;

    writeln!(&mut s, "Total moves:  {:>10}\n", moves)?;

    writeln!(&mut s, "Cache size:   {:>10}", result.stats.cache_size)?;
    writeln!(&mut s, "Cache hits:   {:>10}", result.stats.cache_hits)?;
    writeln!(
        &mut s,
        "Cache ratio:  {:>10.4}\n",
        result.stats.cache_hits as f64 / result.stats.cache_size as f64
    )?;

    for mv in &MOVES {
        write!(&mut s, "{:>6}: ", mv)?;
        match result.move_evaluations.get(mv) {
            Some(eval) => writeln!(&mut s, "{eval:>16.*}", 0, eval = eval)?,
            None => writeln!(&mut s, "{:>16}", "illegal")?,
        }
    }

    if let Some((_, eval)) = result.best_move {
        writeln!(&mut s)?;
        writeln!(&mut s, "  Best: {eval:>16.*}", 0, eval = eval)?;
    }

    writeln!(&mut s)?;

    writeln!(&mut s, "Depth: {}", result.stats.depth)?;
    writeln!(&mut s, "Cutoff probability: {}\n", MIN_PROBABILITY)?;

    writeln!(
        &mut s,
        "Time taken:            {:>8} ms",
        one.num_milliseconds()
    )?;
    writeln!(
        &mut s,
        "Time taken on average: {:>8} ms",
        overall.num_milliseconds() / i64::from(moves)
    )?;
    writeln!(
        &mut s,
        "Time since game start: {:>8} ms\n",
        overall.num_milliseconds()
    )?;

    Ok(s)
}
