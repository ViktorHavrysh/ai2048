extern crate ai2048_lib;

use ai2048_lib::SearchResult;
use ai2048_lib::SearchStatistics;
use ai2048_lib::agent::Agent;
use ai2048_lib::board::Board;
use ai2048_lib::heuristic::composite::CompositeHeuristic;
use ai2048_lib::board::MOVES;

fn main() {
    let heuristic = CompositeHeuristic::default();

    let mut board = Board::default().add_random_tile().add_random_tile();
    let mut agent = Agent::new(board, heuristic, 0.002, 6);
    let mut aggregate_search_statistics = SearchStatistics::default();

    loop {
        let result = agent.make_decision();
        aggregate_search_statistics += result.search_statistics;

        print!("{}[2J", 27 as char);
        println!("{}", build_display(&result, &aggregate_search_statistics));

        match result.best_move {
            Some((mv, _)) => {
                board = board.make_move(mv).add_random_tile();
                agent.update_state(board);
            }
            None => {
                break;
            }
        }
    }

    println!("Game over!");
}

use std::fmt::Write;

fn build_display(result: &SearchResult, aggregate_stats: &SearchStatistics) -> String {
    let mut s = String::new();
    writeln!(&mut s, "{}", result.root_board).unwrap();
    writeln!(&mut s, "{}", result.search_statistics).unwrap();
    writeln!(&mut s, "Total:\n{}", aggregate_stats).unwrap();

    for mv in &MOVES {
        write!(&mut s, "{:?}: ", mv).unwrap();
        match result.move_evaluations.get(mv) {
            Some(eval) => writeln!(&mut s, "{}", eval).unwrap(),
            None => writeln!(&mut s, "invalid").unwrap(),
        }
    }

    if let Some((_, eval)) = result.best_move {
        writeln!(&mut s, "Best: {}", eval).unwrap();
    }

    s
}
