extern crate ai2048;

use ai2048::grid::Grid;
use ai2048::agent::Agent;
use ai2048::heuristic::heat_map::HeatMapHeuristic;
use std::f64;

fn main() {
    let mut grid = Grid::default().add_random_tile().add_random_tile();
    let heuristic = HeatMapHeuristic::new();

    let mut agent = Agent::new(grid, heuristic, 0.004, 6);

    loop {
        println!("{}", grid.to_string());
        let result = agent.make_decision();

        if result.move_evaluations.len() == 0 {
            break;
        }

        let best_evaluttion = result.move_evaluations.values().map(|&v| v).fold(f64::NAN, f64::max);
        let best_move = result.move_evaluations
            .iter()
            .filter(|&(_, &e)| e == best_evaluttion)
            .nth(0)
            .unwrap()
            .0;

        grid = grid.make_move(*best_move).add_random_tile();

        agent.update_state(grid);
    }

    println!("Game over!");
}
