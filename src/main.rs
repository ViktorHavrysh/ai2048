extern crate ai2048;

use ai2048::agent::Agent;
use ai2048::board::Board;
use ai2048::heuristic::composite::CompositeHeuristic;

fn main() {
    let heuristic = CompositeHeuristic::default();

    let mut board = Board::default().add_random_tile().add_random_tile();
    let mut agent = Agent::new(board, heuristic, 0.002, 6);

    loop {
        let result = agent.make_decision();
        print!("{}[2J", 27 as char);
        println!("{}", board);
        println!("{}", result.search_statistics.to_string());

        for mv in ai2048::board::MOVES.iter() {
            println!("{:?}: {}",
                     mv,
                     match result.move_evaluations.get(mv) {
                         Some(eval) => format!("{}", eval),
                         None => "invalid".to_string(),
                     });
        }

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
