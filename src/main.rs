extern crate ai2048;

use ai2048::board::Board;
use ai2048::agent::Agent;
use ai2048::heuristic::heat_map::HeatMapHeuristic;

fn main() {
    let heuristic = HeatMapHeuristic::new();
    let mut board = Board::default().add_random_tile().add_random_tile();
    let mut agent = Agent::new(board, heuristic, 0.004, 6);

    loop {
        let result = agent.make_decision();
        print!("{}[2J", 27 as char);
        println!("{}", board.to_string());
        println!("{}", result.search_statistics.to_string());

        match result.best_move {
            Some((mv, eval)) => {
                println!("Best evaluation: {}", eval);
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
