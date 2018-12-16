#![allow(clippy::needless_pass_by_value)]

use ai2048_lib::game_logic;
use ai2048_lib::searcher::Searcher;
use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use web_sys::console;

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub fn init() {
    set_panic_hook();
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Debug)]
pub enum Move {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
    None = 4,
}

#[wasm_bindgen]
pub fn evaluate_position(grid: Box<[u32]>, min_prob: f32, max_depth: u8) -> Move {
    let grid = transform_grid(&grid);
    let searcher = Searcher::new(min_prob, max_depth);
    let result = searcher.search(grid);
    transform_move(result.best_move)
}

fn transform_grid(grid: &[u32]) -> game_logic::Grid {
    let row0 = [grid[0], grid[4], grid[8], grid[12]];
    let row1 = [grid[1], grid[5], grid[9], grid[13]];
    let row2 = [grid[2], grid[6], grid[10], grid[14]];
    let row3 = [grid[3], grid[7], grid[11], grid[15]];
    let grid = [row0, row1, row2, row3];
    game_logic::Grid::from_human(grid).unwrap()
}

fn transform_move(mv: Option<(game_logic::Move, f32)>) -> Move {
    match mv {
        Some((game_logic::Move::Up, _)) => Move::Up,
        Some((game_logic::Move::Down, _)) => Move::Down,
        Some((game_logic::Move::Left, _)) => Move::Left,
        Some((game_logic::Move::Right, _)) => Move::Right,
        None => {
            console::log_1(&"game over!".into());
            Move::None
        }
    }
}
