use ai2048_lib::game_logic::{Board, MOVES};
use ai2048_lib::searcher::{SearchResult, Searcher};
use chrono::prelude::*;
use futures::Future;
use futures_cpupool::CpuPool;
use std::fmt::{self, Write};
use std::sync::mpsc;

const MIN_PROBABILITY: f32 = 0.0001;
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
        let searcher = Searcher::new(SEARCH_DEPTH, MIN_PROBABILITY);
        let mut board = Board::default().add_random_tile().add_random_tile();
        let start_overall = Utc::now();
        let mut moves = 0;
        loop {
            moves += 1;
            let start_one = Utc::now();
            let result = searcher.search(board);
            let end = Utc::now();
            tx.send(Signal::Display(
                result.clone(),
                moves,
                end - start_one,
                end - start_overall,
            ))?;

            if let Some((mv, _)) = result.best_move {
                board = board.make_move(mv).add_random_tile();
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

    writeln!(&mut s, "{}", result.root_board)?;

    writeln!(&mut s, "Total moves: {:>10}\n", moves)?;

    writeln!(&mut s, "Cache size: {:>10}\n", result.stats.cache_size)?;

    writeln!(
        &mut s,
        "Time taken for one move:            {:>16} ms",
        one.num_milliseconds()
    )?;
    writeln!(
        &mut s,
        "Time taken for one move on average: {:>16} ms",
        overall.num_milliseconds() / moves as i64
    )?;
    writeln!(
        &mut s,
        "Time taken for all moves:           {:>16} ms\n",
        overall.num_milliseconds()
    )?;

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
