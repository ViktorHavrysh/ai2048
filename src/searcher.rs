use search_tree::{ComputerNode, PlayerNode, SearchTree};
use board::{Board, Move};
use std::f64;
use std::collections::HashMap;
use time::{self, Duration};
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

pub struct SearchResult {
    pub root_board: Board,
    pub move_evaluations: HashMap<Move, f64>,
    pub search_statistics: SearchStatistics,
    pub best_move: Option<(Move, f64)>,
}

pub struct SearchStatistics {
    pub search_duration: Duration,
    pub nodes_traversed: usize,
    pub terminal_traversed: usize,
    pub known_player_nodes: usize,
    pub known_computer_nodes: usize,
    pub new_player_nodes: usize,
    pub new_computer_nodes: usize,
}

impl Default for SearchStatistics {
    fn default() -> SearchStatistics {
        SearchStatistics {
            search_duration: Duration::zero(),
            nodes_traversed: 0,
            terminal_traversed: 0,
            known_player_nodes: 0,
            known_computer_nodes: 0,
            new_player_nodes: 0,
            new_computer_nodes: 0,
        }
    }
}

impl SearchStatistics {
    fn known_nodes(&self) -> usize {
        self.known_player_nodes + self.known_computer_nodes
    }
    fn new_nodes(&self) -> usize {
        self.new_player_nodes + self.new_computer_nodes
    }
    fn nodes_per_second(&self) -> u64 {
        (self.nodes_traversed as f64 *
         (1_000_000_000f64 / self.search_duration.num_nanoseconds().unwrap() as f64)) as u64
    }
    fn new_nodes_per_second(&self) -> u64 {
        (self.new_nodes() as f64 *
         (1_000_000_000f64 / self.search_duration.num_nanoseconds().unwrap() as f64)) as u64
    }
}

impl ToString for SearchStatistics {
    fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str("Statistics:\n");
        result.push_str(&format!("Search duration:       {}\n", self.search_duration));
        result.push_str(&format!("Known nodes:           {}\n", self.known_nodes()));
        result.push_str(&format!("Nodes traversed:       {}\n", self.nodes_traversed));
        result.push_str(&format!("New nodes:             {}\n", self.new_nodes()));
        result.push_str(&format!("Terminal nodes:        {}\n", self.terminal_traversed));
        result.push_str(&format!("Nodes per second:      {}\n", self.nodes_per_second()));
        result.push_str(&format!("New nodes per second:  {}\n", self.new_nodes_per_second()));

        result
    }
}

impl<H: Heuristic> Searcher for ExpectiMaxer<H> {
    fn search(&self, search_tree: &SearchTree) -> SearchResult {
        let mut search_statistics = SearchStatistics::default();

        let start = time::now_utc();
        let known_player_nodes_start = search_tree.get_known_player_node_count();
        let known_computer_nodes_start = search_tree.get_known_computer_node_count();

        let hashmap = self.init(search_tree, &mut search_statistics);

        let finish = time::now_utc();
        let elapsed = finish - start;
        let known_player_nodes_finish = search_tree.get_known_player_node_count();
        let known_computer_nodes_finish = search_tree.get_known_computer_node_count();

        search_statistics.search_duration = elapsed;
        search_statistics.new_computer_nodes = known_computer_nodes_finish -
                                               known_computer_nodes_start;
        search_statistics.new_player_nodes = known_player_nodes_finish - known_player_nodes_start;
        search_statistics.known_computer_nodes = known_computer_nodes_finish;
        search_statistics.known_player_nodes = known_player_nodes_finish;

        let best_move = if hashmap.len() > 0 {
            let best_eval = hashmap.values().map(|&v| v).fold(f64::NAN, f64::max);
            let (&mv, &eval) = hashmap.iter()
                .filter(|&(_, &e)| e == best_eval)
                .nth(0)
                .unwrap();

            Some((mv, eval))
        } else {
            None
        };

        SearchResult {
            root_board: search_tree.get_root().get_board().clone(),
            move_evaluations: hashmap,
            search_statistics: search_statistics,
            best_move: best_move,
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

    fn init(&self,
            search_tree: &SearchTree,
            mut search_statistics: &mut SearchStatistics)
            -> HashMap<Move, f64> {
        let children = search_tree.get_root().get_children_by_move();

        if children.len() == 0 {
            return HashMap::new();
        }

        children.iter()
            .map(|(&m, n)| {
                let eval =
                    self.get_computer_node_eval(n,
                                                self.max_search_depth,
                                                1f64,
                                                &mut search_statistics);
                (m, eval)
            })
            .collect()
    }

    fn get_player_node_eval(&self,
                            node: &PlayerNode,
                            depth: u8,
                            probability: f64,
                            mut search_statistics: &mut SearchStatistics)
                            -> f64 {
        search_statistics.nodes_traversed += 1;

        if let Some((prob, avg)) = node.storage.get() {
            if prob >= probability {
                return avg;
            }
        }

        let children = node.get_children_by_move();

        if children.is_empty() || depth == 0 || probability < self.min_probability {
            search_statistics.terminal_traversed += 1;

            let heur = match node.heuristic.get() {
                Some(heur) => heur,
                None => {
                    let heur = self.heuristic.eval(node);
                    node.heuristic.set(Some(heur));
                    heur
                }
            };

            node.storage.set(Some((probability, heur)));

            return heur;
        }

        let avg = children.values()
            .map(|n| self.get_computer_node_eval(n, depth, probability, &mut search_statistics))
            .fold(f64::NAN, f64::max);

        node.storage.set(Some((probability, avg)));

        avg
    }

    fn get_computer_node_eval(&self,
                              node: &ComputerNode,
                              depth: u8,
                              probability: f64,
                              mut search_statistics: &mut SearchStatistics)
                              -> f64 {
        search_statistics.nodes_traversed += 1;
        let children = node.get_children();
        let count = children.with2.len();

        let sum_with2 = children.with2
            .iter()
            .map(|n| {
                self.get_player_node_eval(n,
                                          depth - 1,
                                          probability * PROBABILITY_OF2 / count as f64,
                                          &mut search_statistics)
            })
            .fold(0f64, |acc, x| acc + x);
        let avg_with2 = sum_with2 / count as f64;

        let sum_with4 = children.with4
            .iter()
            .map(|n| {
                self.get_player_node_eval(n,
                                          depth - 1,
                                          probability * PROBABILITY_OF4 / count as f64,
                                          &mut search_statistics)
            })
            .fold(0f64, |acc, x| acc + x);
        let avg_with4 = sum_with4 / count as f64;

        avg_with2 * PROBABILITY_OF2 + avg_with4 * PROBABILITY_OF4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use board::Board;
    use search_tree::SearchTree;
    use heuristic::heat_map::HeatMapHeuristic;

    #[test]
    fn can_get_search_result() {
        let board = Board::default().add_random_tile();
        let search_tree = SearchTree::new(board);
        let heuristic = HeatMapHeuristic::new();
        let searcher = ExpectiMaxer::new(0.01, 3, heuristic);

        let result = searcher.search(&search_tree);

        assert_eq!(result.root_board, board);
        assert!(result.move_evaluations.len() >= 2);
        assert!(result.move_evaluations.len() <= 4);
    }
}
