//! This module intends to provide a lazily-evaluated, cached tree of all possible board states
//! in a 2048 game.
//!
//! The types in this module generate its children only once.
//!
//! They use two different kinds of cache to reduce the amount of computation as much as possible:
//!
//! 1. Each node stores references to its children.
//! 2. When generating the children, the nodes query a `Cache` of known nodes (a transposition
//! table) in case this same node has already been generated through a different set of moves.
//!
//! It achieves this by a combination of interior mutability, reference counted objects and
//! a hashmap.

mod cache;

use board::{self, Board, Move};
use fnv::FnvHashMap;
use lazycell::LazyCell;
use search_tree::cache::Cache;
use std::cell::Cell;
use std::rc::Rc;

/// The `SearchTree` type is the root of the tree of nodes that form all possible board states in
/// a 2048 game. It is the only potentially mutable type in this module. You can generate a new
/// `SearchTree` by providing an initial board state, or use a mutable reference to an existing
/// `SearchTree` to update its root board state in order to reuse nodes already calculated from
/// the previous state.
pub struct SearchTree {
    root_node: Rc<PlayerNode>,
    // I think that, in theory, this cache could be owned by this type, while all its
    // descendats would get a reference to this object, since a `SearchTree` root is expected
    // to outlive all its descendats. However, some of the descendants produce Rc<T> references
    // to nodes, so until I solve that in theory a node can outlive the `SearchTree`, so reference
    // counting it is, for the moment.
    cache: Rc<NodeCache>,
}

struct NodeCache {
    player_node: Cache<Board, PlayerNode>,
    computer_node: Cache<Board, ComputerNode>,
}

impl SearchTree {
    /// Creates a new `SearchTree` from an initial `Board` state.
    pub fn new(board: Board) -> Self {
        let cache = Rc::new(NodeCache {
            player_node: Cache::new(),
            computer_node: Cache::new(),
        });

        let node = cache.player_node
            .get_or_insert_with(board, || PlayerNode::new(board, cache.clone()));

        SearchTree {
            root_node: node,
            cache: cache,
        }
    }

    /// Updates the search tree to have a different root `Board` state. It has an advantage over
    /// creating a new one because it reuses the inner cache of known nodes. This implicitly
    /// invalidates now unreachable board states in the cache (or at least board states that
    /// have no known way to be reached). This also explicitly cleans up the invalidated keys
    /// from the cache.
    pub fn set_root(&mut self, board: Board) {
        let node = self.cache
            .player_node
            .get_or_insert_with(board, || PlayerNode::new(board, self.cache.clone()));

        self.root_node = node;

        self.clean_up_cache();
    }

    /// Gets a reference to the current root node.
    pub fn get_root(&self) -> &PlayerNode {
        self.root_node.as_ref()
    }

    /// Gets the number of known board states that the Player can face on their turn.
    pub fn get_known_player_node_count(&self) -> usize {
        self.cache.player_node.strong_count()
    }

    /// Gets the number of known board states that the Computer can face on its turn.
    pub fn get_known_computer_node_count(&self) -> usize {
        self.cache.computer_node.strong_count()
    }

    fn clean_up_cache(&self) {
        self.cache.player_node.gc();
        self.cache.computer_node.gc();
    }
}

/// This type rerpresents a `Board` state that can be reached on the Player's turn. This type
/// is logically immutable, and there should be no way to create this type from outside the module
/// through any means other than querying the `SearchTree` root and its descendants.
///
/// However, this type makes use of interior mutability to defer generating its children unitl
/// such time as it is asked to do so, and only do it once even then.
pub struct PlayerNode {
    board: Board,
    cache: Rc<NodeCache>,
    children: LazyCell<FnvHashMap<Move, Rc<ComputerNode>>>,
    // This is ugly, because the only reason these are here is that I need them in the searcher.
    // However, I can't think of a less cumbersome way to keep these around and associated with
    // a particular node without the searcher having to keep its own `HashMap` of `Board` states.
    pub heuristic: Cell<Option<f32>>,
}

impl PlayerNode {
    fn new(board: Board, cache: Rc<NodeCache>) -> Self {
        PlayerNode {
            board: board,
            cache: cache,
            children: LazyCell::new(),
            heuristic: Cell::new(None),
        }
    }

    /// Get a reference to the `Board` state associated with this node.
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    /// Returns a `HashMap` of all possible `Move`:`ComputerNode` pairs possible in the current
    /// position. If the `HashMap` it returns is empty, it means Game Over: no possible further
    /// moves by the player!
    pub fn get_children_by_move(&self) -> &FnvHashMap<Move, Rc<ComputerNode>> {
        if let Some(children) = self.children.borrow() {
            children
        } else {
            let children = self.create_children_by_move();
            self.children.fill(children);
            self.children.borrow().unwrap()
        }
    }

    fn create_children_by_move(&self) -> FnvHashMap<Move, Rc<ComputerNode>> {
        let mut children: FnvHashMap<Move, Rc<ComputerNode>> = FnvHashMap::default();

        for &m in &board::MOVES {
            let new_grid = self.board.make_move(m);

            // It is illegal to make a move that doesn't change anything.
            if new_grid != self.board {
                let computer_node = self.cache
                    .computer_node
                    .get_or_insert_with(new_grid,
                                        || ComputerNode::new(new_grid, self.cache.clone()));

                children.insert(m, computer_node);
            }
        }

        children
    }
}

/// This type holds all the children of a computer node. It is useful to separate the children
/// that were generated by spawning a 2 from ones that were spawned with a 4, because in a game
/// of 2048 a 4 only spawns 10% of the time, and it's important to take into account how likely
/// an outcome is.
pub struct ComputerNodeChildren {
    pub with2: Vec<Rc<PlayerNode>>,
    pub with4: Vec<Rc<PlayerNode>>,
}

/// This type rerpresents a `Board` state that can be reached on the Computer's turn. This type
/// is logically immutable, and there should be no way to create this type from outside the moduel
/// through any means other than querying a `PlayerNode`.
///
/// However, this type makes use of interior mutability to defer generating its children unitl
/// such time as it is asked to do so, and only do it once even then.
pub struct ComputerNode {
    board: Board,
    cache: Rc<NodeCache>,
    children: LazyCell<ComputerNodeChildren>,
}

impl ComputerNode {
    fn new(board: Board, cache: Rc<NodeCache>) -> Self {
        ComputerNode {
            board: board,
            cache: cache,
            children: LazyCell::new(),
        }
    }

    /// Get a reference to the `Board` state associated with this node.
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    /// Returns an `ComputerNodeChildren` that represents all possible states that the Player
    /// can face following a computer spawning a random 2 or 4 tile. Can't be empty, by the game'search_tree
    /// logic.

    // It feels like this method should be able to return a `&ComputerNodeChildren`, but I can't
    // think of a way to do it. Oh well.
    pub fn get_children(&self) -> &ComputerNodeChildren {
        {
            if let Some(children) = self.children.borrow() {
                return children;
            } else {
                let children = self.create_children();
                self.children.fill(children);
            }
        }

        self.get_children()
    }

    fn create_children(&self) -> ComputerNodeChildren {
        let children_with2 = self.board
            .get_possible_boards_with2()
            .map(|board| {
                self.cache
                    .player_node
                    .get_or_insert_with(board, || PlayerNode::new(board, self.cache.clone()))
            })
            .collect::<Vec<_>>();

        let children_with4 = self.board
            .get_possible_boards_with4()
            .map(|board| {
                self.cache
                    .player_node
                    .get_or_insert_with(board, || PlayerNode::new(board, self.cache.clone()))
            })
            .collect::<Vec<_>>();

        debug_assert!(children_with2.len() != 0);
        debug_assert!(children_with4.len() != 0);

        ComputerNodeChildren {
            with2: children_with2,
            with4: children_with4,
        }
    }
}

#[cfg(test)]
mod tests {

    use board::{Board, Move};

    use std::collections::{HashMap, HashSet};
    use super::*;

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

        let actual = player_node.get_children_by_move();

        for (key, value) in expected {
            assert_eq!(value, *actual.get(&key).unwrap().get_board());
        }

        assert_eq!(1, search_tree.get_known_player_node_count());
        assert_eq!(4, search_tree.get_known_computer_node_count());
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_get_computernode_children() {
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
