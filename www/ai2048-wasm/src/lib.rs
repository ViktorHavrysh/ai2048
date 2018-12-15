use ai2048_lib::game_logic::{self, Board};
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
pub fn evaluate_position(board: Box<[u32]>) -> u8 {
    let row0 = [board[0], board[4], board[8], board[12]];
    let row1 = [board[1], board[5], board[9], board[13]];
    let row2 = [board[2], board[6], board[10], board[14]];
    let row3 = [board[3], board[7], board[11], board[15]];
    let board = [row0, row1, row2, row3];
    let board = Board::from_human(board).unwrap();
    console::log_1(&format!("{:?}", board).into());
    console::log_1(&format!("{}", board).into());
    let searcher = Searcher::new(0.0001);
    let result = searcher.search(board);
    console::log_1(&format!("{:?}", result).into());
    let mv = match result.best_move {
        Some((game_logic::Move::Up, _)) => Move::Up,
        Some((game_logic::Move::Down, _)) => Move::Down,
        Some((game_logic::Move::Left, _)) => Move::Left,
        Some((game_logic::Move::Right, _)) => Move::Right,
        None => Move::None,
    };
    console::log_1(&format!("{:?}", mv).into());

    mv as u8
}

// Called by our JS entry point to run the example.
#[wasm_bindgen]
pub fn run() {
    set_panic_hook();
}
