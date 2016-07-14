mod cache;

use grid::{self, Grid, Move};
use search_tree::cache::Cache;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

struct NodeCache {
    player_node: Cache<Grid, PlayerNode>,
    computer_node: Cache<Grid, ComputerNode>,
}

pub struct SearchTree {
    root_node: Rc<PlayerNode>,
    cache: Rc<NodeCache>,
}

impl SearchTree {
    pub fn new(grid: Grid) -> SearchTree {
        let player_node_cache = Cache::new();
        let computer_node_cache = Cache::new();

        let cache = Rc::new(NodeCache {
            player_node: player_node_cache,
            computer_node: computer_node_cache,
        });

        let node = cache.player_node
            .get_or_insert_with(grid, || PlayerNode::new(grid, cache.clone()));

        SearchTree {
            root_node: node,
            cache: cache,
        }
    }

    pub fn set_root(&mut self, grid: Grid) {
        let node = self.cache
            .player_node
            .get_or_insert_with(grid, || PlayerNode::new(grid, self.cache.clone()));

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
    grid: Grid,
    cache: Rc<NodeCache>,
    children: RefCell<Option<Rc<HashMap<Move, Rc<ComputerNode>>>>>,
}

impl PlayerNode {
    pub fn get_grid(&self) -> &Grid {
        &self.grid
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

    fn new(grid: Grid, cache: Rc<NodeCache>) -> PlayerNode {
        PlayerNode {
            grid: grid,
            cache: cache,
            children: RefCell::new(None),
        }
    }

    fn create_children_by_move(&self) -> HashMap<Move, Rc<ComputerNode>> {
        let mut children: HashMap<Move, Rc<ComputerNode>> = HashMap::new();

        for m in grid::MOVES.iter() {
            let new_grid = self.grid.make_move(*m);

            if new_grid != self.grid {
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

pub struct ComputerNode {
    grid: Grid,
    cache: Rc<NodeCache>,
    children: RefCell<Option<Vec<Rc<PlayerNode>>>>,
}

impl ComputerNode {
    fn new(grid: Grid, cache: Rc<NodeCache>) -> ComputerNode {
        ComputerNode {
            grid: grid,
            cache: cache,
            children: RefCell::new(None),
        }
    }

    pub fn get_grid(&self) -> &Grid {
        &self.grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use grid::{Grid, Move};

    use std::collections::HashMap;

    #[test]
    fn can_create_new_searchtree() {
        let expected_grid = Grid::default().add_random_tile();
        let search_tree = SearchTree::new(expected_grid);
        let actual_grid = *search_tree.get_root().get_grid();

        assert_eq!(expected_grid, actual_grid);
    }

    #[test]
    fn can_set_new_root() {
        let grid1 = Grid::default().add_random_tile();
        let grid2 = Grid::default().add_random_tile().add_random_tile();
        let mut search_tree = SearchTree::new(grid1);

        search_tree.set_root(grid2);

        assert_eq!(grid2, *search_tree.get_root().get_grid());
        assert_eq!(1, search_tree.get_known_player_node_count());
        let total = search_tree.cache.player_node.len();
        assert_eq!(1, total);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_get_playernode_children_by_move() {
        // arrange
        let grid = Grid::new(&[
            [0, 0, 0, 2],
            [0, 2, 0, 2],
            [4, 0, 0, 2],
            [0, 0, 0, 2]
        ]).unwrap();

        let search_tree = SearchTree::new(grid);

        let player_node = search_tree.get_root();

        let mut expected = HashMap::new();
        expected.insert(Move::Left, Grid::new(&[
            [2, 0, 0, 0],
            [4, 0, 0, 0],
            [4, 2, 0, 0],
            [2, 0, 0, 0]
        ]).unwrap());
        expected.insert(Move::Right, Grid::new(&[
            [0, 0, 0, 2],
            [0, 0, 0, 4],
            [0, 0, 4, 2],
            [0, 0, 0, 2]
        ]).unwrap());
        expected.insert(Move::Up, Grid::new(&[
            [4, 2, 0, 4],
            [0, 0, 0, 4],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]).unwrap());
        expected.insert(Move::Down, Grid::new(&[
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
}
