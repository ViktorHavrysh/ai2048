mod cache;

use grid::{self, Grid, Move};
use search_tree::cache::Cache;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct SearchTree {
    root_node: Rc<PlayerNode>,
    player_node_cache: Rc<Cache<Grid, PlayerNode>>,
    commputer_node_cache: Rc<Cache<Grid, ComputerNode>>,
}

impl SearchTree {
    pub fn new(grid: Grid) -> SearchTree {
        let player_node_cache = Rc::new(Cache::new());
        let computer_node_cache = Rc::new(Cache::new());

        let node = player_node_cache.get_or_insert_with(grid, || {
            PlayerNode::new(grid, player_node_cache.clone(), computer_node_cache.clone())
        });

        SearchTree {
            root_node: node,
            player_node_cache: player_node_cache,
            commputer_node_cache: computer_node_cache,
        }
    }

    pub fn set_root(&mut self, grid: Grid) {
        let node = self.player_node_cache.get_or_insert_with(grid, || {
            PlayerNode::new(grid,
                            self.player_node_cache.clone(),
                            self.commputer_node_cache.clone())
        });

        self.root_node = node;

        self.clean_up_cache();
    }

    pub fn get_root(&self) -> Rc<PlayerNode> {
        self.root_node.clone()
    }

    pub fn get_known_player_node_count(&self) -> usize {
        self.player_node_cache.strong_count()
    }

    pub fn get_known_computer_node_count(&self) -> usize {
        self.commputer_node_cache.strong_count()
    }

    fn clean_up_cache(&self) {
        self.player_node_cache.gc();
        self.commputer_node_cache.gc();
    }
}

pub struct PlayerNode {
    grid: Grid,
    children: RefCell<Option<Rc<HashMap<Move, Rc<ComputerNode>>>>>,
    player_node_cache: Rc<Cache<Grid, PlayerNode>>,
    commputer_node_cache: Rc<Cache<Grid, ComputerNode>>,
}

impl PlayerNode {
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

    pub fn get_grid(&self) -> &Grid {
        &self.grid
    }

    fn new(grid: Grid,
           player_node_cache: Rc<Cache<Grid, PlayerNode>>,
           commputer_node_cache: Rc<Cache<Grid, ComputerNode>>)
           -> PlayerNode {
        PlayerNode {
            grid: grid,
            player_node_cache: player_node_cache,
            commputer_node_cache: commputer_node_cache,
            children: RefCell::new(None),
        }
    }

    fn create_children_by_move(&self) -> HashMap<Move, Rc<ComputerNode>> {
        let mut children: HashMap<Move, Rc<ComputerNode>> = HashMap::new();

        for m in &grid::MOVES {
            let new_grid = self.grid.make_move(*m);

            if new_grid != self.grid {
                let computer_node = self.commputer_node_cache
                    .get_or_insert_with(new_grid, || {
                        ComputerNode::new(new_grid,
                                          self.player_node_cache.clone(),
                                          self.commputer_node_cache.clone())
                    });

                children.insert(*m, computer_node);
            }
        }

        children
    }
}

pub struct ComputerNode {
    grid: Grid,
    children: RefCell<Option<Rc<Vec<Rc<ComputerNode>>>>>,
    player_node_cache: Rc<Cache<Grid, PlayerNode>>,
    commputer_node_cache: Rc<Cache<Grid, ComputerNode>>,
}

impl ComputerNode {
    pub fn new(grid: Grid,
               player_node_cache: Rc<Cache<Grid, PlayerNode>>,
               commputer_node_cache: Rc<Cache<Grid, ComputerNode>>)
               -> ComputerNode {
        ComputerNode {
            grid: grid,
            children: RefCell::new(None),
            player_node_cache: player_node_cache,
            commputer_node_cache: commputer_node_cache,
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
    fn can_set_root_node() {}

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
        let total = search_tree.player_node_cache.len();
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
