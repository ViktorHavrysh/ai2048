//! `Grid` represents the board state in a 2048 game.
//!
//! `Grid` saves its state as a 4x4 array of `u8` values.
//!
//! To cram the value of a cell into into one byte of memory, `Grid` uses a logarithmic
//! representation of the value displayed to the player. That is, `2` becomes `1`,
//! `4` becomes `2`, `8` becomes `3`, etc. The maximum cell value theoretically achievable in a
//! standard game of 2048 is `65,536`, and that is represented by the value `16`, so a byte is
//! more than enough storage for a single cell. `0` stays a `0`.

#[derive(Eq, PartialEq, Debug)]
pub struct Grid {
    grid: [[u8; 4]; 4],
}

pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

impl Grid {
    /// Creates a new `Grid` from an array of human input.
    pub fn new(grid: &[[u32; 4]; 4]) -> Option<Grid> {
        let mut result = [[0u8; 4]; 4];

        for x in 0..4 {
            for y in 0..4 {
                let log = parse_to_logspace(grid[x][y]);

                match log {
                    Some(l) => {
                        result[x][y] = l;
                    }
                    None => {
                        return None;
                    }
                }
            }
        }

        Some(Grid { grid: result })
    }

    /// Creates a `Grid` with every cell empty (represented by the value `0`).
    pub fn empty() -> Grid {
        let result = [[0u8; 4]; 4];

        Grid { grid: result }
    }

    /// Gets a reference to the inner representation of the `Grid`, which is a 4x4 array of `u8`.
    pub fn get_grid(&self) -> &[[u8; 4]; 4] {
        &self.grid
    }

    /// Returns a reference of the inner representation of the `Grid` as a flat array of `u8`.
    pub fn flatten(&self) -> &[u8; 16] {
        use std::mem;
        unsafe { mem::transmute(&self.grid) }
    }

    /// Adds a random tile (10% a `2`, 90% a `4`) to a random empty cell on the board.
    pub fn add_random_tile(&self) -> Grid {
        use std::mem;
        use rand;
        use rand::Rng;

        let mut rng = rand::thread_rng();

        let mut flat = self.flatten().clone();

        let empty_cell_count = flat.iter().filter(|&&v| v == 0).count();

        let create_four = rng.gen_weighted_bool(10);
        let value = if create_four {1} else {2};
        let mut position = rng.gen_range(0, empty_cell_count);

        for x in 0..16
        {
            if flat[x] != 0 {
                continue;
            }

            if position == 0 {
                flat[x] = value;
                break;
            }

            position = position - 1;
        }

        let grid: [[u8; 4]; 4] = unsafe { mem::transmute(flat) };

        Grid { grid: grid }
    }

    /// Returns a `Grid` that would result from making a certain `Move` in the current state.
    pub fn make_move(&self, mv: Move) -> Grid {
        match mv {
            Move::Left  => self.move_left(),
            Move::Right => self.move_right(),
            Move::Up    => self.move_up(),
            Move::Down  => self.move_down()
        }
    }

    /// Returns all possible `Grid`s that can result of the computer spawning a `2` in a random
    /// empty cell.
    pub fn get_possible_grids_with2(&self) -> Vec<Grid> {
        self.get_possible_grids(1)
    }

    /// Returns all possible `Grid`s that can result of the computer spawning a `2` in a random
    /// empty cell.
    pub fn get_possible_grids_with4(&self) -> Vec<Grid> {
        self.get_possible_grids(2)
    }

    fn get_possible_grids(&self, new_value: u8) -> Vec<Grid> {
        let mut result = Vec::<Grid>::new();

        for x in 0..4 {
            for y in 0..4 {
                if self.grid[x][y] != 0 {
                    continue;
                }

                let mut possible_grid = self.grid.clone();
                possible_grid[x][y] = new_value;
                result.push(Grid { grid: possible_grid });
            }
        }

        result
    }

    fn move_left(&self) -> Grid {
        let mut result = [[0; 4]; 4];

        for x in 0..4 {
            let mut last = 0;
            let mut last_index = 0;

            for y in 0..4 {
                let current = self.grid[x][y];

                if current == 0 {
                    continue;
                }

                if last == 0 {
                    last = current;
                    continue;
                }

                if current == last {
                    result[x][last_index] = last + 1;
                    last = 0;
                    last_index = last_index + 1;
                    continue;
                }

                result[x][last_index] = last;
                last = current;
                last_index = last_index + 1;
            }

            if last != 0 {
                result[x][last_index] = last;
            }
        }

        Grid { grid: result }
    }

    fn move_right(&self) -> Grid {
        let mut result = [[0; 4]; 4];

        for x in 0..4 {
            let mut last = 0;
            let mut last_index = 3;

            for y in (0..4).rev() {
                let current = self.grid[x][y];

                if current == 0 {
                    continue;
                }

                if last == 0 {
                    last = current;
                    continue;
                }

                if current == last {
                    result[x][last_index] = last + 1;
                    last = 0;
                    last_index = last_index - 1;
                    continue;
                }

                result[x][last_index] = last;
                last = current;
                last_index = last_index - 1;
            }

            if last != 0 {
                result[x][last_index] = last;
            }
        }

        Grid { grid: result }
    }

    fn move_up(&self) -> Grid {
        let mut result = [[0; 4]; 4];

        for y in 0..4 {
            let mut last = 0;
            let mut last_index = 0;

            for x in 0..4 {
                let current = self.grid[x][y];

                if current == 0 {
                    continue;
                }

                if last == 0 {
                    last = current;
                    continue;
                }

                if current == last {
                    result[last_index][y] = last + 1;
                    last = 0;
                    last_index = last_index + 1;
                    continue;
                }

                result[last_index][y] = last;
                last = current;
                last_index = last_index + 1;
            }

            if last != 0 {
                result[last_index][y] = last;
            }
        }

        Grid { grid: result }
    }

    fn move_down(&self) -> Grid {
        let mut result = [[0; 4]; 4];

        for y in 0..4 {
            let mut last = 0;
            let mut last_index = 3;

            for x in (0..4).rev() {
                let current = self.grid[x][y];

                if current == 0 {
                    continue;
                }

                if last == 0 {
                    last = current;
                    continue;
                }

                if current == last {
                    result[last_index][y] = last + 1;
                    last = 0;
                    last_index = last_index - 1;
                    continue;
                }

                result[last_index][y] = last;
                last = current;
                last_index = last_index - 1;
            }

            if last != 0 {
                result[last_index][y] = last;
            }
        }

        Grid { grid: result }
    }
}

impl ToString for Grid {
    fn to_string(&self) -> String {
        let mut s = String::new();

        for x in 0..4 {
            for y in 0..4 {
                let human = get_human(self.grid[x][y]);
                let human = format!("{number:>width$}", number = human, width = 6);
                s.push_str(&human);
            }
            s.push('\n');
        }

        s
    }
}

fn get_human(n: u8) -> u32 {
    match n {
        0 => 0,
        _ => 1 << n
    }
}

fn parse_to_logspace(n: u32) -> Option<u8> {
    use std::f64;

    let log = match n {
        0 => 0f64,
        _ => (n as f64).log2()
    };

    if (log.round() - log) < 1e-10 {
        Some(log.round() as u8)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_empty_grid() {
        let expected = Grid{ grid: [
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
            ] };

        let actual = Grid::empty();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_create_grid_from_human_input() {
        let expected: [[u8; 4]; 4] = [
                [0, 1, 2, 3],
                [4, 5, 6, 7],
                [8, 9, 10, 11],
                [12, 13, 14, 15]
            ];

        let actual = Grid::new(&[
                [0, 2, 4, 8],
                [16, 32, 64, 128],
                [256, 512, 1024, 2048],
                [4096, 8192, 16384, 32768]
            ]);

        assert!(actual.is_some());
        assert_eq!(&expected, actual.unwrap().get_grid());
    }

    #[test]
    fn can_return_none_on_invalid_input() {
        let result = Grid::new(&[
                [0, 1, 2, 3],
                [4, 5, 6, 7],
                [8, 9, 10, 11],
                [12, 13, 14, 15]
            ]);

        assert!(result.is_none());
    }

    #[test]
    fn can_add_random_tile() {
        let grid = Grid::empty().add_random_tile();

        let count = grid
            .flatten()
            .iter()
            .filter(|&&v| v == 1 || v == 2)
            .count();

        assert_eq!(1, count);
    }

    #[test]
    fn can_to_string() {
        // arrange
        let grid = Grid::new(&[
                [0, 2, 4, 8],
                [16, 32, 64, 128],
                [256, 512, 1024, 2048],
                [4096, 8192, 16384, 32768]
            ]).unwrap();

        let mut expected = String::new();
        expected.push_str("     0     2     4     8\n");
        expected.push_str("    16    32    64   128\n");
        expected.push_str("   256   512  1024  2048\n");
        expected.push_str("  4096  8192 16384 32768\n");

        // act
        let actual = grid.to_string();

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_left() {
        // arrange
        let grid = Grid::new(&[
                [2, 2, 4, 4],
                [0, 2, 2, 0],
                [0, 2, 2, 2],
                [2, 0, 0, 2]
            ]).unwrap();
        let expected = Grid::new(&[
                [4, 8, 0, 0],
                [4, 0, 0, 0],
                [4, 2, 0, 0],
                [4, 0, 0, 0]
            ]).unwrap();

        // act
        let actual = grid.make_move(Move::Left);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_right() {
        // arrange
        let grid = Grid::new(&[
                [2, 2, 4, 4],
                [0, 2, 2, 0],
                [0, 2, 2, 2],
                [2, 0, 0, 2]
            ]).unwrap();
        let expected = Grid::new(&[
                [0, 0, 4, 8],
                [0, 0, 0, 4],
                [0, 0, 2, 4],
                [0, 0, 0, 4]
            ]).unwrap();

        // act
        let actual = grid.make_move(Move::Right);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_up() {
        // arrange
        let grid = Grid::new(&[
                [2, 2, 4, 4],
                [0, 2, 2, 0],
                [0, 2, 2, 2],
                [2, 0, 0, 2]
            ]).unwrap();
        let expected = Grid::new(&[
                [4, 4, 4, 4],
                [0, 2, 4, 4],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ]).unwrap();

        // act
        let actual = grid.make_move(Move::Up);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_down() {
        // arrange
        let grid = Grid::new(&[
                [2, 2, 4, 4],
                [0, 2, 2, 0],
                [0, 2, 2, 2],
                [2, 0, 0, 2]
            ]).unwrap();
        let expected = Grid::new(&[
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 2, 4, 4],
                [4, 4, 4, 4]
            ]).unwrap();

        // act
        let actual = grid.make_move(Move::Down);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_get_possible_grids_with2() {
        // arrange
        let grid = Grid::new(&[
                [0, 8, 8, 8],
                [8, 8, 0, 8],
                [8, 8, 8, 0],
                [8, 0, 8, 8]
            ]).unwrap();

        let expected = vec![
            Grid::new(&[
                [2, 8, 8, 8],
                [8, 8, 0, 8],
                [8, 8, 8, 0],
                [8, 0, 8, 8]
            ]).unwrap(),
            Grid::new(&[
                [0, 8, 8, 8],
                [8, 8, 2, 8],
                [8, 8, 8, 0],
                [8, 0, 8, 8]
            ]).unwrap(),
            Grid::new(&[
                [0, 8, 8, 8],
                [8, 8, 0, 8],
                [8, 8, 8, 2],
                [8, 0, 8, 8]
            ]).unwrap(),
            Grid::new(&[
                [0, 8, 8, 8],
                [8, 8, 0, 8],
                [8, 8, 8, 0],
                [8, 2, 8, 8]
            ]).unwrap()];

        // act
        let actual = grid.get_possible_grids_with2();

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_get_possible_grids_with4() {
        // arrange
        let grid = Grid::new(&[
                [0, 8, 8, 8],
                [8, 8, 0, 8],
                [8, 8, 8, 0],
                [8, 0, 8, 8]
            ]).unwrap();

        let expected = vec![
            Grid::new(&[
                [4, 8, 8, 8],
                [8, 8, 0, 8],
                [8, 8, 8, 0],
                [8, 0, 8, 8]
            ]).unwrap(),
            Grid::new(&[
                [0, 8, 8, 8],
                [8, 8, 4, 8],
                [8, 8, 8, 0],
                [8, 0, 8, 8]
            ]).unwrap(),
            Grid::new(&[
                [0, 8, 8, 8],
                [8, 8, 0, 8],
                [8, 8, 8, 4],
                [8, 0, 8, 8]
            ]).unwrap(),
            Grid::new(&[
                [0, 8, 8, 8],
                [8, 8, 0, 8],
                [8, 8, 8, 0],
                [8, 4, 8, 8]
            ]).unwrap()];

        // act
        let actual = grid.get_possible_grids_with4();

        // assert
        assert_eq!(expected, actual);
    }
}
