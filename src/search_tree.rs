use grid;
use grid::Grid;
use grid::Move;

use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub struct SearchTree {
    root_node: PlayerNode,

    known_player_nodes: HashMap<Grid, Weak<PlayerNode>>,
    known_computer_nodes: HashMap<Grid, Weak<ComputerNode>>,
}

pub struct PlayerNode {
    grid: Grid,
    children: RefCell<Option<HashMap<Move, Rc<ComputerNode>>>>
}

impl PlayerNode {
    pub fn new(grid: Grid) -> PlayerNode {
        PlayerNode {
            grid: grid,
            children: RefCell::new(None)
        }
    }

    pub fn get_children_by_move(&self) -> HashMap<Move, Rc<ComputerNode>> {
        {
            let mut cache = self.children.borrow_mut();
            if cache.is_some() {
                return cache.as_ref().unwrap().clone();
            }

            let children = self.create_children_by_move();
            *cache = Some(children);
        }

        self.get_children_by_move()
    }

    fn create_children_by_move(&self) -> HashMap<Move, Rc<ComputerNode>> {
        let mut children: HashMap<Move, Rc<ComputerNode>> = HashMap::new();

        for m in grid::MOVES {
            let new_grid = self.grid.make_move(*m);

            if new_grid != self.grid {
                let computer_node = ComputerNode::new(new_grid);
                children.insert(m.clone(), Rc::new(computer_node));
            }
        }

        children
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct ComputerNode {
    grid: Grid,
}

impl ComputerNode {
    pub fn new(grid: Grid) -> ComputerNode {
        ComputerNode {
            grid: grid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use grid::{Grid, Move};

    use std::rc::Rc;
    use std::collections::HashMap;

    #[test]
    fn can_create_new_playernode() {
        let grid = Grid::empty().add_random_tile();
        let node = PlayerNode::new(grid);

        assert_eq!(grid, node.grid);
    }

    #[test]
    fn can_get_playernode_children_by_move() {
        // arrange
        let grid = Grid::new(&[
            [0, 0, 0, 2],
            [0, 2, 0, 2],
            [4, 0, 0, 2],
            [0, 0, 0, 2]
        ]).unwrap();
        let node = PlayerNode::new(grid);

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
        let actual = node.create_children_by_move();

        // assert
        assert_eq!(expected, actual);
    }
}