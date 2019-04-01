use ai2048_lib::game_logic::{GameEngine, Grid, MOVES};
use ai2048_lib::searcher::{self, SearchResult};
use cfg_if::cfg_if;
use chrono::prelude::*;
use chrono::Duration;
use futures::Future;
use futures_cpupool::CpuPool;
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::sync::mpsc;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        #[global_allocator]
        static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
    }
}

const MIN_PROBABILITY: f32 = 0.001;

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
                    let entry = times.entry(result.depth).or_insert((0, Duration::zero()));
                    *entry = (entry.0 + 1, entry.1 + one);
                    println!("{}", build_display(&result, moves, one, overall, &times)?);
                }
            };
        }
        println!("Game over!");
        Ok(())
    });

    let compute_loop = pool.spawn_fn(move || {
        let game_engine = GameEngine::new();
        let mut grid = Grid::default().add_random_tile().add_random_tile();
        let start_overall = Utc::now();
        let mut moves = 0;
        loop {
            moves += 1;
            let start_one = Utc::now();
            let result = searcher::search(grid, MIN_PROBABILITY);
            let end = Utc::now();
            tx.send(Signal::Display(
                result.clone(),
                moves,
                end - start_one,
                end - start_overall,
            ))?;

            if let Some(mv) = result.best_move {
                grid = game_engine.make_move(grid, mv).add_random_tile();
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

    writeln!(&mut s, "Depth: {}", result.depth)?;
    writeln!(&mut s, "Cutoff probability: {}", MIN_PROBABILITY)?;

    writeln!(&mut s)?;

    writeln!(
        &mut s,
        "Time taken:             {:>8.3} ms",
        one.num_nanoseconds().unwrap() as f32 / 1_000_000.0f32
    )?;
    writeln!(
        &mut s,
        "Nodes traveled:         {:>8} ({:>2.0}ns/node)",
        result.stats.nodes,
        one.num_nanoseconds().unwrap() as f32 / result.stats.nodes as f32
    )?;
    writeln!(
        &mut s,
        "In cache:               {:>8} [{:>4.1}%]",
        result.stats.cache_size,
        f64::from(result.stats.cache_size) * 100.0 / f64::from(result.stats.nodes)
    )?;
    writeln!(&mut s, "Evaluated by:")?;
    writeln!(
        &mut s,
        "Cached value:           {:>8} [{:>4.1}%]",
        result.stats.cache_hits,
        f64::from(result.stats.cache_hits) * 100.0 / f64::from(result.stats.nodes)
    )?;
    writeln!(
        &mut s,
        "Heuristic:              {:>8} [{:>4.1}%]",
        result.stats.evals,
        f64::from(result.stats.evals) * 100.0 / f64::from(result.stats.nodes)
    )?;
    writeln!(
        &mut s,
        "Averaging over children:{:>8} [{:>4.1}%]",
        result.stats.average,
        f64::from(result.stats.average) * 100.0 / f64::from(result.stats.nodes)
    )?;

    writeln!(&mut s)?;

    writeln!(
        &mut s,
        "DEPTH |   TOTAL TIME, ms |          MOVES | AVG TIME, ms"
    )?;
    writeln!(
        &mut s,
        "------+------------------+----------------+-------------"
    )?;
    for depth in searcher::MIN_DEPTH..=searcher::MAX_DEPTH {
        let (moves_d, time) = times.get(&depth).cloned().unwrap_or((0, Duration::zero()));
        let time_avg = match time.num_milliseconds() as f32 / moves_d as f32 {
            nan if nan.is_nan() => String::default(),
            not_nan => format!("{:12.3}", not_nan),
        };
        writeln!(
            &mut s,
            "{:>5} | {:>8} [{:>4.1}%] | {:>5}  [{:>4.1}%] | {}",
            depth,
            time.num_milliseconds(),
            time.num_milliseconds() as f64 * 100.0 / overall.num_milliseconds() as f64,
            moves_d,
            f64::from(moves_d) * 100.0 / f64::from(moves),
            time_avg
        )?;
    }
    writeln!(
        &mut s,
        "------+------------------+----------------+-------------"
    )?;
    writeln!(
        &mut s,
        "TOTAL | {:>8}         | {:>5} ({:>5.1}/s)| {:12.3}",
        overall.num_milliseconds(),
        moves,
        moves as f32 * 1000.0 / (overall.num_milliseconds() as f32),
        overall.num_milliseconds() as f32 / moves as f32
    )?;

    Ok(s)
}
