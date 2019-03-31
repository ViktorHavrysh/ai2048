use ai2048_lib::game_logic::Grid;
use ai2048_lib::searcher::{self, SearchResult};
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use lazy_static::lazy_static;

const MIN_PROBABILITY: f32 = 0.0001;

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

criterion_group! {
    name = large_sample;
    config = Criterion::default().sample_size(10);
    targets = single_moves
}

criterion_main!(large_sample);
