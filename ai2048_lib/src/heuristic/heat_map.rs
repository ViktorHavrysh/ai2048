use super::Heuristic;
use std::f32;
use board::Board;
use search_tree::PlayerNode;

type HeatMap = [[i64; 4]; 4];

const MIN: f32 = -10_000_000_000f32;

const EMPTY_CELLS_WITHOUT_PENALTY: i64 = 6;

#[cfg_attr(rustfmt, rustfmt_skip)]
const HEAT_MAP: HeatMap = [
    [0,   0,   1,   3],
    [0,   1,   2,   4],
    [2,   3,   4,   6],
    [5,   8,  16, 128]];

pub struct HeatMapHeuristic {
    heat_maps: [HeatMap; 8],
}

impl Heuristic for HeatMapHeuristic {
    fn eval(&self, node: &PlayerNode) -> f32 {
        if node.children_by_move().len() == 0 {
            return MIN;
        }

        let mut result =
            self.heat_maps.iter().map(|&h| evaluate_board(node.board(), h)).max().unwrap();

        let empty_cell_evaluation = evaluate_empty_cells(node.board());

        if empty_cell_evaluation < EMPTY_CELLS_WITHOUT_PENALTY {
            result -= 1 << (EMPTY_CELLS_WITHOUT_PENALTY - empty_cell_evaluation);
        }

        result as f32
    }
}

fn evaluate_board(board: &Board, heat_map: HeatMap) -> i64 {
    let grid = board.grid();
    let mut result = 0;

    for x in 0..4 {
        for y in 0..4 {
            result += (1 << (grid[x][y] * 2)) * heat_map[x][y]
        }
    }

    result
}

fn evaluate_empty_cells(board: &Board) -> i64 {
    let mut adjacent_count = 0;
    {
        let grid = board.grid();

        for y in 0..4 {
            let mut x = 0;
            while x < 3 {
                if grid[x][y] != 0 && grid[x][y] == grid[x + 1][y] {
                    adjacent_count += 1;
                    x += 2;
                } else {
                    x += 1;
                }
            }
        }

        for x in 0..4 {
            let mut y = 0;
            while y < 3 {
                if grid[x][y] != 0 && grid[x][y] == grid[x][y + 1] {
                    adjacent_count += 1;
                    y += 2;
                } else {
                    y += 1;
                }
            }
        }
    }

    let empty_count = board.flatten().iter().filter(|&&x| x == 0).count() as i64;

    adjacent_count + empty_count
}

impl HeatMapHeuristic {
    pub fn new() -> HeatMapHeuristic {
        let mut heat_maps = [[[0; 4]; 4]; 8];
        heat_maps[0] = HEAT_MAP;

        for x in 1..4 {
            heat_maps[x] = HeatMapHeuristic::rotate_cw(&heat_maps[x - 1]);
        }

        for x in 0..4 {
            heat_maps[x + 4] = HeatMapHeuristic::mirror(&heat_maps[x]);
        }

        HeatMapHeuristic { heat_maps }
    }

    fn rotate_cw(heat_map: &HeatMap) -> HeatMap {
        let mut new = [[0; 4]; 4];
        for x in 0..4 {
            for y in 0..4 {
                new[y][3 - x] = heat_map[x][y];
            }
        }

        new
    }

    fn mirror(heat_map: &HeatMap) -> HeatMap {
        let mut new = [[0; 4]; 4];
        for x in 0..4 {
            for y in 0..4 {
                new[y][x] = heat_map[x][y]
            }
        }

        new
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use std::f32;
    use std::collections::HashSet;
    use board::Board;
    use search_tree::SearchTree;

    #[test]
    fn can_create_heuristic() {
        let heur = HeatMapHeuristic::new();

        println!("{:?}", heur.heat_maps);
        assert_eq!(super::HEAT_MAP, heur.heat_maps[0]);
        assert_eq!(super::HEAT_MAP[3][3], heur.heat_maps[1][3][0]);
        assert_eq!(super::HEAT_MAP[3][3], heur.heat_maps[2][0][0]);
        assert_eq!(super::HEAT_MAP[3][3], heur.heat_maps[3][0][3]);
        assert_eq!(8, heur.heat_maps.iter().collect::<HashSet<_>>().len());
    }

    #[test]
    fn can_eval() {
        let heur = HeatMapHeuristic::new();

        let board = Board::default().add_random_tile().add_random_tile();
        let search_tree = SearchTree::new(board);

        let eval = heur.eval(search_tree.root());

        assert_ne!(eval, f32::NAN);
    }
}
