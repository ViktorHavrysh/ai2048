use ai2048_lib::game_logic::{Grid, Move};
use ai2048_lib::heuristic;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use lazy_static::lazy_static;

lazy_static! {
    static ref TEST_GRID: Grid =
        Grid::from_human([[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
}

fn make_move(c: &mut Criterion) {
    c.bench_function("move left", |b| b.iter(|| TEST_GRID.make_move(Move::Left)));
    c.bench_function("move right", |b| {
        b.iter(|| TEST_GRID.make_move(Move::Right))
    });
    c.bench_function("move up", |b| b.iter(|| TEST_GRID.make_move(Move::Up)));
    c.bench_function("move down", |b| b.iter(|| TEST_GRID.make_move(Move::Down)));
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
    c.bench_function("random moves with 2", |b| {
        b.iter(|| consume_iter(TEST_GRID.ai_moves_with2()))
    });
    c.bench_function("random moves with 4", |b| {
        b.iter(|| consume_iter(TEST_GRID.ai_moves_with4()))
    });
    c.bench_function("player moves", |b| {
        b.iter(|| consume_iter(TEST_GRID.player_moves()))
    });
}

fn game_over(c: &mut Criterion) {
    c.bench_function("game over", |b| b.iter(|| TEST_GRID.game_over()));
}

fn heuristic(c: &mut Criterion) {
    c.bench_function("eval", |b| b.iter(|| heuristic::eval(*TEST_GRID)));
}

criterion_group! {
    name = huge_sample;
    config = Criterion::default().sample_size(50);
    targets = make_move, transpose, count_empty, possible_moves, game_over, heuristic
}

criterion_main!(huge_sample);
