use ai2048_lib::game_logic::{GameEngine, Grid, Move};
use ai2048_lib::heuristic::Heuristic;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use lazy_static::lazy_static;

lazy_static! {
    static ref TEST_GRID: Grid =
        Grid::from_human([[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
}

fn make_move(c: &mut Criterion) {
    let game_engine = GameEngine::new();
    c.bench_function("move left", move |b| {
        b.iter(|| game_engine.make_move(*TEST_GRID, Move::Left))
    });
    c.bench_function("move right", move |b| {
        b.iter(|| game_engine.make_move(*TEST_GRID, Move::Right))
    });
    c.bench_function("move up", move |b| {
        b.iter(|| game_engine.make_move(*TEST_GRID, Move::Up))
    });
    c.bench_function("move down", move |b| {
        b.iter(|| game_engine.make_move(*TEST_GRID, Move::Down))
    });
}

fn transpose(c: &mut Criterion) {
    c.bench_function("transpose", |b| b.iter(|| TEST_GRID.transpose()));
}

fn count_empty(c: &mut Criterion) {
    c.bench_function("count empty", |b| b.iter(|| TEST_GRID.count_empty()));
}

fn consume_iter<T>(iter: impl Iterator<Item = T>) {
    for item in iter {
        criterion::black_box(item);
    }
}

fn possible_moves(c: &mut Criterion) {
    let game_engine = GameEngine::new();
    c.bench_function("random moves with 2", move |b| {
        b.iter(|| consume_iter(game_engine.random_moves_with2(*TEST_GRID)));
    });
    c.bench_function("random moves with 4", move |b| {
        b.iter(|| consume_iter(game_engine.random_moves_with4(*TEST_GRID)))
    });
    c.bench_function("player moves", move |b| {
        b.iter(|| consume_iter(game_engine.player_moves(*TEST_GRID)))
    });
}

fn game_over(c: &mut Criterion) {
    let game_engine = GameEngine::new();
    c.bench_function("game over", move |b| {
        b.iter(|| game_engine.game_over(*TEST_GRID))
    });
}

fn heuristic(c: &mut Criterion) {
    let heuristic = Heuristic::new();
    c.bench_function("eval", move |b| b.iter(|| heuristic.eval(*TEST_GRID)));
}

criterion_group! {
    name = huge_sample;
    config = Criterion::default().sample_size(50);
    targets = make_move, transpose, count_empty, possible_moves, game_over, heuristic
}

criterion_main!(huge_sample);
