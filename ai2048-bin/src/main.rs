use ai2048_lib::game_logic::{Grid, MOVES};
use ai2048_lib::searcher::{SearchResult, Searcher};
use chrono::prelude::*;
use chrono::Duration;
use futures::Future;
use futures_cpupool::CpuPool;
use std::alloc::System;
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::sync::mpsc;

const MIN_PROBABILITY: f32 = 0.0005;
const MAX_DEPTH: u8 = 12;

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
        let mut times: HashMap<u8, (i32, chrono::Duration)> = HashMap::new();
        loop {
            let message = rx.recv()?;

            match message {
                Signal::Stop => break,
                Signal::Display(result, moves, one, overall) => {
                    let entry = times
                        .entry(result.stats.depth)
                        .or_insert((0, Duration::zero()));
                    *entry = (entry.0 + 1, entry.1 + one);
                    println!("{}", build_display(&result, moves, one, overall, &times)?);
                }
            };
        }
        println!("Game over!");
        Ok(())
    });

    let compute_loop = pool.spawn_fn(move || {
        let searcher = Searcher::new(MIN_PROBABILITY, MAX_DEPTH);
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

            if let Some(mv) = result.best_move {
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
    times: &HashMap<u8, (i32, chrono::Duration)>,
) -> Result<String, fmt::Error> {
    let mut s = String::new();
    write!(&mut s, "{}[2J", 27 as char)?; // clear screen

    writeln!(&mut s, "{}", result.root_grid)?;

    for mv in &MOVES {
        write!(&mut s, "{:>8}: ", mv)?;
        match result.move_evaluations.get(mv) {
            Some(eval) => writeln!(&mut s, "{eval:>16.*}", 0, eval = eval)?,
            None => writeln!(&mut s, "{:>16}", "")?,
        }
    }

    writeln!(&mut s)?;

    writeln!(&mut s, "Depth: {}", result.stats.depth)?;
    writeln!(&mut s, "Cutoff probability: {}\n", MIN_PROBABILITY)?;

    writeln!(&mut s, "Cache size:     {:>10}", result.stats.cache_size)?;
    writeln!(&mut s, "Cache hits:     {:>10}", result.stats.cache_hits)?;
    writeln!(
        &mut s,
        "Cache ratio:    {:>10.4}\n",
        result.stats.cache_hits as f64 / result.stats.cache_size as f64
    )?;

    writeln!(&mut s, "Total moves:    {:>10}", moves)?;
    writeln!(
        &mut s,
        "Moves per second: {:>8.2}\n",
        f64::from(moves) / (overall.num_seconds() as f64)
    )?;

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

    for depth in 3u8..=MAX_DEPTH {
        let (moves, time) = times.get(&depth).cloned().unwrap_or((0, Duration::zero()));
        writeln!(
            &mut s,
            "Depth {:>2}: {:>8} ms, {:>4} moves, {:8.1} ms/move",
            depth,
            time.num_milliseconds(),
            moves,
            time.num_milliseconds() as f64 / moves as f64
        )?;
    }

    Ok(s)
}
