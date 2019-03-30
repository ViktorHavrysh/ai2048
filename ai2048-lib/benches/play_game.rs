use ai2048_lib::game_logic::Grid;
use ai2048_lib::searcher::{self, SearchResult};
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use lazy_static::lazy_static;

const MIN_PROBABILITY: f32 = 0.0001;

fn play_moves(start: Grid, moves: u32) -> SearchResult {
    let mut grid = start;
    let mut result = SearchResult::default();
    for _ in 0..moves {
        result = searcher::search(grid, MIN_PROBABILITY);
        if let Some(mv) = result.best_move {
            grid = grid.make_move(mv).add_random_tile();
        } else {
            break;
        }
    }
    result
}

fn calc_move(start: Grid) -> SearchResult {
    searcher::search(start, MIN_PROBABILITY)
}

lazy_static! {
    static ref GRID_DEFAULT: Grid = Grid::default().add_random_tile();
    static ref GRID_DEPTH_8: Grid = {
        let grid = Grid::from_human([
            [8, 2, 4, 2],
            [32, 32, 4, 2],
            [512, 128, 64, 2],
            [1024, 256, 16, 0],
        ])
        .expect("couldn't parse obviously correct grid");
        assert_eq!(searcher::search(grid, 0.1).depth, 8);
        grid
    };
}

fn single_moves(c: &mut Criterion) {
    c.bench_function("depth 8 move", move |b| b.iter(|| calc_move(*GRID_DEPTH_8)));
}

fn multiple_moves(c: &mut Criterion) {
    c.bench_function("100 moves post depth 8", |b| {
        b.iter(|| play_moves(*GRID_DEPTH_8, 100))
    });
}

criterion_group! {
    name = large_sample;
    config = Criterion::default().sample_size(10);
    targets = single_moves
}

criterion_group! {
    name = small_sample;
    config = Criterion::default().sample_size(3);
    targets = multiple_moves
}

criterion_main!(large_sample, small_sample);
