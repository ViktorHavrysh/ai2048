use search_tree::{SearchTree, PlayerNode, ComputerNode};

use grid::{Grid, Move};

use std::f64;

use std::collections::HashMap;

const PROBABILITY_OF2: f64 = 0.9;
const PROBABILITY_OF4: f64 = 0.1;

pub struct Searcher<'a, F: Fn(&Grid) -> f64> {
    search_tree: &'a SearchTree,
    min_probability: f64,
    max_search_depth: u8,
    heuristic: F,
}

pub struct SearchResult {
    pub root_grid: Grid,
    pub move_evaluations: HashMap<Move, f64>,
}

impl<'a, F: Fn(&Grid) -> f64> Searcher<'a, F> {
    pub fn new(search_tree: &'a SearchTree,
               min_probability: f64,
               max_search_depth: u8,
               heuristic: F)
               -> Searcher<'a, F> {
        Searcher {
            search_tree: search_tree,
            min_probability: min_probability,
            max_search_depth: max_search_depth,
            heuristic: heuristic,
        }
    }

    fn init(&self) -> HashMap<Move, f64> {
        self.search_tree
            .get_root()
            .get_children_by_move()
            .iter()
            .map(|(&m, n)| (m, self.get_computer_node_eval(n, self.max_search_depth, 1f64)))
            .collect::<HashMap<Move, f64>>()
    }

    fn get_player_node_eval(&self, node: &PlayerNode, depth: u8, probability: f64) -> f64 {
        assert!(depth != 0);

        let children = node.get_children_by_move();

        if children.len() == 0 || depth == 0 || probability < self.min_probability {
            if let Some(heur) = node.heuristic.get() {
                return heur;
            }

            let heur = (self.heuristic)(node.get_grid());
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

trait FloatIterExt {
    fn float_min(&mut self) -> f64;
    fn float_max(&mut self) -> f64;
}

impl<T> FloatIterExt for T
    where T: Iterator<Item = f64>
{
    fn float_max(&mut self) -> f64 {
        self.fold(f64::NAN, f64::max)
    }

    fn float_min(&mut self) -> f64 {
        self.fold(f64::NAN, f64::min)
    }
}

#[cfg(test)]
mod tests {
    use super::Searcher;
    use grid::Grid;
    use search_tree::SearchTree;

    #[test]
    fn can_create_searcher() {
        let grid = Grid::default().add_random_tile();
        let search_tree = SearchTree::new(grid);

        let searcher = Searcher::new(&search_tree, 0.002, 10, |g| g.flatten().len() as f64);

        assert_eq!(16.0, (searcher.heuristic)(&grid));
    }
}
