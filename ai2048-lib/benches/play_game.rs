use ai2048_lib::game_logic::Grid;
use ai2048_lib::searcher::{SearchResult, Searcher};
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use lazy_static::lazy_static;

const MIN_PROBABILITY: f32 = 0.0001;
const MAX_DEPTH: u8 = 12;

fn play_moves(start: Grid, moves: u32) -> SearchResult {
    let searcher = Searcher::new(MIN_PROBABILITY, MAX_DEPTH);
    let mut grid = start;
    let mut result = SearchResult::default();
    for _ in 0..moves {
        result = searcher.search(grid);
        if let Some(mv) = result.best_move {
            grid = grid.make_move(mv).add_random_tile();
        } else {
            break;
        }
    }
    result
}

fn calc_move(start: Grid) -> SearchResult {
    let searcher = Searcher::new(MIN_PROBABILITY, MAX_DEPTH);
    searcher.search(start)
}

lazy_static! {
    static ref GRID_DEFAULT: Grid = Grid::default().add_random_tile();
    static ref GRID_DEPTH_8: Grid = Grid::from_human([
        [8, 2, 4, 2],
        [32, 32, 4, 2],
        [512, 128, 64, 2],
        [1024, 256, 16, 0],
    ])
    .expect("couldn't parse obviously correct grid");
}

fn get_test_grid(depth: u8) -> Grid {
    let grid = match depth {
        8 => *GRID_DEPTH_8,
        _ => panic!("No grid prepared for this depth"),
    };
    assert_eq!(Searcher::new(0.1, 16).search(grid).stats.depth, depth);
    grid
}

fn single_moves(c: &mut Criterion) {
    c.bench_function("depth 8 move", move |b| {
        b.iter(|| calc_move(get_test_grid(8)))
    });
}

fn multiple_moves(c: &mut Criterion) {
    c.bench_function("100 moves post depth 8", |b| {
        b.iter(|| play_moves(get_test_grid(8), 100))
    });
}

criterion_group! {
    name = large_sample;
    config = Criterion::default().sample_size(20);
    targets = single_moves
}

criterion_group! {
    name = small_sample;
    config = Criterion::default().sample_size(5);
    targets = multiple_moves
}

criterion_main!(large_sample, small_sample);
