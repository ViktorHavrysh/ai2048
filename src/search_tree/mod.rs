mod cache;

use board::{self, Board, Move};
use search_tree::cache::Cache;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{Cell, RefCell};

struct NodeCache {
    player_node: Cache<Board, PlayerNode>,
    computer_node: Cache<Board, ComputerNode>,
}

pub struct SearchTree {
    root_node: Rc<PlayerNode>,
    cache: Rc<NodeCache>,
}

impl SearchTree {
    pub fn new(board: Board) -> SearchTree {
        let player_node_cache = Cache::new();
        let computer_node_cache = Cache::new();

        let cache = Rc::new(NodeCache {
            player_node: player_node_cache,
            computer_node: computer_node_cache,
        });

        let node = cache.player_node
            .get_or_insert_with(board, || PlayerNode::new(board, cache.clone()));

        SearchTree {
            root_node: node,
            cache: cache,
        }
    }

    pub fn set_root(&mut self, board: Board) {
        let node = self.cache
            .player_node
            .get_or_insert_with(board, || PlayerNode::new(board, self.cache.clone()));

        self.root_node = node;

        self.clean_up_cache();
    }

    pub fn get_root(&self) -> Rc<PlayerNode> {
        self.root_node.clone()
    }

    pub fn get_known_player_node_count(&self) -> usize {
        self.cache.player_node.strong_count()
    }

    pub fn get_known_computer_node_count(&self) -> usize {
        self.cache.computer_node.strong_count()
    }

    fn clean_up_cache(&self) {
        self.cache.player_node.gc();
        self.cache.computer_node.gc();
    }
}

pub struct PlayerNode {
    board: Board,
    cache: Rc<NodeCache>,
    children: RefCell<Option<Rc<HashMap<Move, Rc<ComputerNode>>>>>,
    // This is ugly, because the only reason these are here is that I need them in the searcher.
    // However, I can't think of a less cumbersome way to keep these around and associated with
    // a particular node
    pub heuristic: Cell<Option<f64>>,
    pub storage: Cell<Option<(f64, f64)>>,
}

impl PlayerNode {
    fn new(board: Board, cache: Rc<NodeCache>) -> PlayerNode {
        PlayerNode {
            board: board,
            cache: cache,
            children: RefCell::new(None),
            heuristic: Cell::new(None),
            storage: Cell::new(None),
        }
    }

    pub fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn get_children_by_move(&self) -> Rc<HashMap<Move, Rc<ComputerNode>>> {
        {
            let mut cached_children = self.children.borrow_mut();
            if cached_children.is_some() {
                return cached_children.as_ref().unwrap().clone();
            }

            let children = self.create_children_by_move();
            *cached_children = Some(Rc::new(children));
        }

        self.get_children_by_move()
    }

    fn create_children_by_move(&self) -> HashMap<Move, Rc<ComputerNode>> {
        let mut children: HashMap<Move, Rc<ComputerNode>> = HashMap::new();

        for m in board::MOVES.iter() {
            let new_grid = self.board.make_move(*m);

            if new_grid != self.board {
                let computer_node = self.cache
                    .computer_node
                    .get_or_insert_with(new_grid,
                                        || ComputerNode::new(new_grid, self.cache.clone()));

                children.insert(*m, computer_node);
            }
        }

        children
    }
}

pub struct ComputerNodeChildren {
    pub with2: Vec<Rc<PlayerNode>>,
    pub with4: Vec<Rc<PlayerNode>>,
}

pub struct ComputerNode {
    board: Board,
    cache: Rc<NodeCache>,
    children: RefCell<Option<Rc<ComputerNodeChildren>>>,
}

impl ComputerNode {
    fn new(board: Board, cache: Rc<NodeCache>) -> ComputerNode {
        ComputerNode {
            board: board,
            cache: cache,
            children: RefCell::new(None),
        }
    }

    pub fn get_grid(&self) -> &Board {
        &self.board
    }

    pub fn get_children(&self) -> Rc<ComputerNodeChildren> {
        {
            let mut cached_children = self.children.borrow_mut();
            if cached_children.is_some() {
                return cached_children.as_ref().unwrap().clone();
            }

            let children = self.create_children();
            *cached_children = Some(Rc::new(children));
        }

        self.get_children()
    }

    fn create_children(&self) -> ComputerNodeChildren {
        let children_with2 = self.board
            .get_possible_boards_with2()
            .iter()
            .map(|&g| {
                self.cache
                    .player_node
                    .get_or_insert_with(g, || PlayerNode::new(g, self.cache.clone()))
            })
            .collect();

        let children_with4 = self.board
            .get_possible_boards_with4()
            .iter()
            .map(|&g| {
                self.cache
                    .player_node
                    .get_or_insert_with(g, || PlayerNode::new(g, self.cache.clone()))
            })
            .collect();

        ComputerNodeChildren {
            with2: children_with2,
            with4: children_with4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use board::{Board, Move};

    use std::collections::{HashMap, HashSet};

    #[test]
    fn can_create_new_searchtree() {
        let expected_grid = Board::default().add_random_tile();
        let search_tree = SearchTree::new(expected_grid);
        let actual_grid = *search_tree.get_root().get_board();

        assert_eq!(expected_grid, actual_grid);
    }

    #[test]
    fn can_set_new_root() {
        let grid1 = Board::default().add_random_tile();
        let grid2 = Board::default().add_random_tile().add_random_tile();
        let mut search_tree = SearchTree::new(grid1);

        search_tree.set_root(grid2);

        assert_eq!(grid2, *search_tree.get_root().get_board());
        assert_eq!(1, search_tree.get_known_player_node_count());
        let total = search_tree.cache.player_node.len();
        assert_eq!(1, total);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_get_playernode_children_by_move() {
        // arrange
        let board = Board::new(&[
            [0, 0, 0, 2],
            [0, 2, 0, 2],
            [4, 0, 0, 2],
            [0, 0, 0, 2]
        ]).unwrap();

        let search_tree = SearchTree::new(board);

        let player_node = search_tree.get_root();

        let mut expected = HashMap::new();
        expected.insert(Move::Left, Board::new(&[
            [2, 0, 0, 0],
            [4, 0, 0, 0],
            [4, 2, 0, 0],
            [2, 0, 0, 0]
        ]).unwrap());
        expected.insert(Move::Right, Board::new(&[
            [0, 0, 0, 2],
            [0, 0, 0, 4],
            [0, 0, 4, 2],
            [0, 0, 0, 2]
        ]).unwrap());
        expected.insert(Move::Up, Board::new(&[
            [4, 2, 0, 4],
            [0, 0, 0, 4],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]).unwrap());
        expected.insert(Move::Down, Board::new(&[
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 4],
            [4, 2, 0, 4]
        ]).unwrap());

        // act
        let actual = player_node.get_children_by_move();

        // assert
        for (key, value) in expected {
            assert_eq!(value, *actual.get(&key).unwrap().get_grid());
        }

        assert_eq!(1, search_tree.get_known_player_node_count());
        assert_eq!(4, search_tree.get_known_computer_node_count());
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_get_computernode_children() {
        // arrange
        let board = Board::new(&[
            [0, 2, 4, 2],
            [0, 4, 2, 4],
            [4, 2, 4, 2],
            [2, 4, 2, 4]
        ]).unwrap();
        let search_tree = SearchTree::new(board);

        // two possible moves: up and left
        // up:   [4, 2, 4, 2],
        //       [2, 4, 2, 4],
        //       [0, 2, 4, 2],
        //       [0, 4, 2, 4]
        //
        // left: [2, 4, 2, 0],
        //       [4, 2, 4, 0],
        //       [4, 2, 4, 2],
        //       [2, 4, 2, 4]

        // this leads to 8 possible child nodes:
        let mut expected_with2 = HashSet::new();
        expected_with2.insert(Board::new(&[
            [4, 2, 4, 2],
            [2, 4, 2, 4],
            [2, 2, 4, 2],
            [0, 4, 2, 4]
        ]).unwrap());
        expected_with2.insert(Board::new(&[
            [4, 2, 4, 2],
            [2, 4, 2, 4],
            [0, 2, 4, 2],
            [2, 4, 2, 4]
        ]).unwrap());
        expected_with2.insert(Board::new(&[
            [2, 4, 2, 2],
            [4, 2, 4, 0],
            [4, 2, 4, 2],
            [2, 4, 2, 4]
        ]).unwrap());
        expected_with2.insert(Board::new(&[
            [2, 4, 2, 0],
            [4, 2, 4, 2],
            [4, 2, 4, 2],
            [2, 4, 2, 4]
        ]).unwrap());

        let mut expected_with4 = HashSet::new();
        expected_with4.insert(Board::new(&[
            [2, 4, 2, 4],
            [4, 2, 4, 0],
            [4, 2, 4, 2],
            [2, 4, 2, 4]
        ]).unwrap());
        expected_with4.insert(Board::new(&[
            [2, 4, 2, 0],
            [4, 2, 4, 4],
            [4, 2, 4, 2],
            [2, 4, 2, 4]
        ]).unwrap());
        expected_with4.insert(Board::new(&[
            [4, 2, 4, 2],
            [2, 4, 2, 4],
            [4, 2, 4, 2],
            [0, 4, 2, 4]
        ]).unwrap());
        expected_with4.insert(Board::new(&[
            [4, 2, 4, 2],
            [2, 4, 2, 4],
            [0, 2, 4, 2],
            [4, 4, 2, 4]
        ]).unwrap());

        // act
        let actual_with2 = search_tree.get_root()
            .get_children_by_move()
            .values()
            .flat_map(|v| v.get_children().with2.clone())
            .map(|n| n.get_board().clone())
            .collect::<HashSet<_>>();

        let actual_with4 = search_tree.get_root()
            .get_children_by_move()
            .values()
            .flat_map(|v| v.get_children().with4.clone())
            .map(|n| n.get_board().clone())
            .collect::<HashSet<_>>();

        assert_eq!(expected_with2, actual_with2);
        assert_eq!(expected_with4, actual_with4);
    }
}
