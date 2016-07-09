use grid::Grid;

pub struct SearchTree {
    root_node: PlayerNode,
    heuristic: Box<Fn(Grid) -> f64>,

    known_player_nodes: HashMap<u32, HashMap<Grid, PlayerNode>>,
    known_computer_nodes: HashMap<u32, HashMap<Grid, PlayerNode>>,
}

impl SearchTree {
}

struct PlayerNode {
    grid: Grid,
    sum: Option<u32>,
    search_tree: &SearchTree,
}

struct ComputerNode {
    grid: Grid,
    search_tree: &SearchTree,
    sum: Option<u32>,
}