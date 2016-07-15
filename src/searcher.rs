use search_tree::{SearchTree, PlayerNode, ComputerNode};
use grid::{Grid, Move};
use std::collections::HashMap;
use float_ext::FloatIterExt;
use heuristic::Heuristic;

const PROBABILITY_OF2: f64 = 0.9;
const PROBABILITY_OF4: f64 = 0.1;

pub trait Searcher {
    fn search(&self, search_tree: &SearchTree) -> SearchResult;
}

pub struct ExpectiMaxer<H: Heuristic> {
    min_probability: f64,
    max_search_depth: u8,
    heuristic: H,
}

#[derive(Debug)]
pub struct SearchResult {
    pub root_grid: Grid,
    pub move_evaluations: HashMap<Move, f64>,
}

impl<H: Heuristic> Searcher for ExpectiMaxer<H> {
    fn search(&self, search_tree: &SearchTree) -> SearchResult {
        let grid = search_tree.get_root().get_grid().clone();
        let hashmap = self.init(search_tree);

        SearchResult {
            root_grid: grid,
            move_evaluations: hashmap,
        }
    }
}

impl<H: Heuristic> ExpectiMaxer<H> {
    pub fn new(min_probability: f64, max_search_depth: u8, heuristic: H) -> ExpectiMaxer<H> {
        assert!(max_search_depth != 0);
        ExpectiMaxer {
            min_probability: min_probability,
            max_search_depth: max_search_depth,
            heuristic: heuristic,
        }
    }

    fn init(&self, search_tree: &SearchTree) -> HashMap<Move, f64> {
        search_tree.get_root()
            .get_children_by_move()
            .iter()
            .map(|(&m, n)| (m, self.get_computer_node_eval(n, self.max_search_depth, 1f64)))
            .collect::<HashMap<Move, f64>>()
    }

    fn get_player_node_eval(&self, node: &PlayerNode, depth: u8, probability: f64) -> f64 {
        let children = node.get_children_by_move();

        if children.len() == 0 || depth == 0 || probability < self.min_probability {
            if let Some(heur) = node.heuristic.get() {
                return heur;
            }

            let heur = self.heuristic.eval(node);
            node.heuristic.set(Some(heur));

            return heur;
        }

        children.values().map(|n| self.get_computer_node_eval(n, depth, probability)).float_max()
    }

    fn get_computer_node_eval(&self, node: &ComputerNode, depth: u8, probability: f64) -> f64 {
        let children = node.get_children();
        let count = children.with2().len();

        let sum_with2 = children.with2()
            .iter()
            .map(|n| {
                self.get_player_node_eval(n,
                                          depth - 1,
                                          probability * PROBABILITY_OF2 / count as f64)
            })
            .fold(0f64, |acc, x| acc + x);
        let avg_with2 = sum_with2 / count as f64;

        let sum_with4 = children.with4()
            .iter()
            .map(|n| {
                self.get_player_node_eval(n,
                                          depth - 1,
                                          probability * PROBABILITY_OF2 / count as f64)
            })
            .fold(0f64, |acc, x| acc + x);
        let avg_with4 = sum_with4 / count as f64;

        avg_with2 * PROBABILITY_OF2 + avg_with4 * PROBABILITY_OF4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grid::Grid;
    use search_tree::SearchTree;
    use heuristic::heat_map::HeatMapHeuristic;

    #[test]
    fn can_get_search_result() {
        let grid = Grid::default().add_random_tile();
        let search_tree = SearchTree::new(grid);
        let heuristic = HeatMapHeuristic::new();
        let searcher = ExpectiMaxer::new(0.01, 3, heuristic);

        let result = searcher.search(&search_tree);

        assert_eq!(result.root_grid, grid);
        assert!(result.move_evaluations.len() >= 2);
        assert!(result.move_evaluations.len() <= 4);
    }
}
