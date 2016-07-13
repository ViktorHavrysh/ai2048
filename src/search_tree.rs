use grid::{self, Grid, Move};

use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub struct SearchTree {
    root_node: RefCell<Option<Rc<PlayerNode>>>,
    known_player_nodes: RefCell<HashMap<Grid, Weak<PlayerNode>>>,
    known_computer_nodes: RefCell<HashMap<Grid, Weak<ComputerNode>>>,
}

impl SearchTree {
    fn empty() -> SearchTree {
        SearchTree {
            root_node: RefCell::new(None),
            known_player_nodes: RefCell::new(HashMap::new()),
            known_computer_nodes: RefCell::new(HashMap::new()),
        }
    }

    pub fn create_from_initial_state(grid: Grid) -> Rc<SearchTree> {
        let search_tree_strong = Rc::new(SearchTree::empty());
        let search_tree_weak = Rc::downgrade(&search_tree_strong);

        let node = Rc::new(PlayerNode::new(grid, search_tree_weak));

        search_tree_strong.set_root(node);

        search_tree_strong
    }

    pub fn set_root(&self, root_node: Rc<PlayerNode>) {
        let mut known_player_nodes = self.known_player_nodes.borrow_mut();
        known_player_nodes.entry(*root_node.get_grid())
            .or_insert_with(|| Rc::downgrade(&root_node));

        let mut root_node_mut = self.root_node.borrow_mut();
        *root_node_mut = Some(root_node);
    }

    pub fn get_root(&self) -> Rc<PlayerNode> {
        self.root_node.borrow().as_ref().unwrap().clone()
    }
}

pub struct PlayerNode {
    grid: Grid,
    search_tree: Weak<SearchTree>,
    children: RefCell<Option<Rc<HashMap<Move, Rc<ComputerNode>>>>>,
}

impl PlayerNode {
    fn new(grid: Grid, search_tree: Weak<SearchTree>) -> PlayerNode {
        PlayerNode {
            grid: grid,
            search_tree: search_tree,
            children: RefCell::new(None),
        }
    }

    pub fn get_children_by_move(&self) -> Rc<HashMap<Move, Rc<ComputerNode>>> {
        {
            let mut cache = self.children.borrow_mut();
            if cache.is_some() {
                return cache.as_ref().unwrap().clone();
            }

            let children = self.create_children_by_move();
            *cache = Some(Rc::new(children));
        }

        self.get_children_by_move()
    }

    fn create_children_by_move(&self) -> HashMap<Move, Rc<ComputerNode>> {
        let mut children: HashMap<Move, Rc<ComputerNode>> = HashMap::new();

        let search_tree = self.search_tree.upgrade().unwrap();
        let mut known_computer_nodes = search_tree.known_computer_nodes.borrow_mut();

        for m in &grid::MOVES {
            let new_grid = self.grid.make_move(*m);

            if new_grid != self.grid {
                let computer_node = match known_computer_nodes.get(&new_grid)
                    .and_then(|n| n.upgrade()) {
                    Some(computer_node) => computer_node,
                    None => {
                        let computer_node = Rc::new(ComputerNode::new(new_grid));
                        known_computer_nodes.insert(new_grid, Rc::downgrade(&computer_node));
                        computer_node
                    }
                };

                children.insert(*m, computer_node);
            }
        }

        children
    }

    pub fn get_grid(&self) -> &Grid {
        &self.grid
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct ComputerNode {
    grid: Grid,
}

impl ComputerNode {
    pub fn new(grid: Grid) -> ComputerNode {
        ComputerNode { grid: grid }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use grid::{Grid, Move};

    use std::rc::Rc;
    use std::collections::HashMap;

    #[test]
    fn can_set_root_node() {}

    #[test]
    fn can_create_new_searchtree() {
        let expected_grid = Grid::empty().add_random_tile();
        let search_tree = SearchTree::create_from_initial_state(expected_grid);
        let actual_grid = *search_tree.get_root().get_grid();

        assert_eq!(expected_grid, actual_grid);
    }

    #[test]
    fn can_set_new_root() {
        let grid1 = Grid::empty().add_random_tile();
        let search_tree1 = SearchTree::create_from_initial_state(grid1);
        let grid2 = Grid::empty().add_random_tile().add_random_tile();
        let search_tree2 = SearchTree::create_from_initial_state(grid2);

        let node1 = search_tree1.get_root();
        search_tree2.set_root(node1);

        let root1 = search_tree1.get_root();
        let actual1 = root1.get_grid();

        let root2 = search_tree2.get_root();
        let actual2 = root2.get_grid();

        assert_eq!(actual1, actual2);
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

        let search_tree = SearchTree::create_from_initial_state(grid);

        let player_node = search_tree.get_root();

        let mut expected = HashMap::new();
        expected.insert(Move::Left, Rc::new(ComputerNode::new(Grid::new(&[
            [2, 0, 0, 0],
            [4, 0, 0, 0],
            [4, 2, 0, 0],
            [2, 0, 0, 0]
        ]).unwrap())));
        expected.insert(Move::Right, Rc::new(ComputerNode::new(Grid::new(&[
            [0, 0, 0, 2],
            [0, 0, 0, 4],
            [0, 0, 4, 2],
            [0, 0, 0, 2]
        ]).unwrap())));
        expected.insert(Move::Up, Rc::new(ComputerNode::new(Grid::new(&[
            [4, 2, 0, 4],
            [0, 0, 0, 4],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]).unwrap())));
        expected.insert(Move::Down, Rc::new(ComputerNode::new(Grid::new(&[
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 4],
            [4, 2, 0, 4]
        ]).unwrap())));

        // act
        let actual = player_node.get_children_by_move();

        // assert
        assert_eq!(expected, *actual);
    }
}
