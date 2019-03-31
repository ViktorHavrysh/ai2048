use ai2048_lib::game_logic::Grid;
use ai2048_lib::searcher;
use chrono::prelude::*;
use chrono::Duration;
use itertools::Itertools;
use rayon::prelude::*;
use std::sync::Mutex;

const MIN_PROBABILITY: f32 = 0.001;
const TOTAL_RUNS: usize = 100;

fn main() {
    let finished = Mutex::new(0);
    let started = Mutex::new(0);

    let start = Utc::now();

    println!("MIN_PROBABILITY: {}", MIN_PROBABILITY);

    let mut results = (0..TOTAL_RUNS)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|_| {
            let started = {
                let mut started = started.lock().unwrap();
                *started += 1;
                *started
            };
            let run_result = run_one();
            let finished = {
                let mut finished = finished.lock().unwrap();
                *finished += 1;
                *finished
            };
            println!(
                "Result #{:>3} ({:>3}): Finished: {:>4} sec; Survived: {:>5} moves; {:>4.1} ms per move; Biggest tile: {:>5}",
                finished,
                started,
                run_result.elapsed.num_seconds(),
                run_result.moves,
                run_result.per_move().num_microseconds().unwrap() as f64 / 1000.0,
                run_result.biggest,
            );
            run_result
        })
        .collect::<Vec<_>>();

    let elapsed = Utc::now() - start;

    results.sort_by_key(|result| -i64::from(result.biggest));
    let grouped_by_biggest: Vec<(u32, usize)> = results
        .iter()
        .group_by(|result| result.biggest)
        .into_iter()
        .map(|(biggest, group)| (biggest, group.count()))
        .collect();
    let avg_moves =
        results.iter().map(|result| result.moves).sum::<u32>() as f32 / TOTAL_RUNS as f32;
    let avg_elapsed = results
        .iter()
        .map(|result| result.elapsed)
        .fold(Duration::zero(), |a, b| a + b)
        / TOTAL_RUNS as i32;
    let mut agg_count = 0;
    for (biggest, count) in grouped_by_biggest {
        agg_count += count;
        println!(
            "{:>5}: {:>5.1}%",
            biggest,
            (agg_count * 100) as f32 / TOTAL_RUNS as f32
        );
    }
    println!("Average moves: {}", avg_moves);
    println!("Average duration: {}", avg_elapsed);
    println!(
        "The whole test took {} min {} sec",
        elapsed.num_minutes(),
        elapsed.num_seconds() % 60
    );
}

struct RunResult {
    moves: u32,
    biggest: u32,
    elapsed: Duration,
}

impl RunResult {
    fn per_move(&self) -> Duration {
        self.elapsed / (self.moves as i32)
    }
}

fn run_one() -> RunResult {
    let mut grid = Grid::default().add_random_tile().add_random_tile();
    let start_overall = Utc::now();
    let mut moves = 0;
    loop {
        moves += 1;
        let result = searcher::search(grid, MIN_PROBABILITY);
        if let Some(mv) = result.best_move {
            grid = grid.make_move(mv).add_random_tile();
        } else {
            let elapsed = Utc::now() - start_overall;
            let biggest = grid.biggest_tile();
            return RunResult {
                moves,
                biggest,
                elapsed,
            };
        }
    }
}
