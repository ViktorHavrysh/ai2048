//! `Board` represents the board state in a 2048 game.
//!
//! `Board` saves its state as a 4x4 array of `u8` values.
//!
//! To cram the value of a cell into into one byte of memory, `Board` uses a logarithmic
//! representation of the value displayed to the player. That is, `2` becomes `1`,
//! `4` becomes `2`, `8` becomes `3`, etc. The maximum cell value theoretically achievable in a
//! standard game of 2048 is `65,536`, and that is represented by the value `16`, so a byte is
//! more than enough storage for a single cell. `0` stays a `0`.
//!
//! `Board`, in general, encodes all the rules of the game: it can generate new states
//! given a move a player makes, or all possible states following the computer spwaning a random
//! tile. Unsurprisingly, in order to write an AI for a game, the AI needs an emulation of the
//! game itself.
use std::fmt;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Default)]
pub struct Board {
    grid: [[u8; 4]; 4],
}

/// Represents a move.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

/// All possible moves.
pub const MOVES: [Move; 4] = [Move::Left, Move::Right, Move::Up, Move::Down];

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();

        for row in &self.grid {
            for val in row {
                let human = get_human(*val);
                let human = format!("{number:>width$}", number = human, width = 6);
                s.push_str(&human);
            }
            s.push('\n');
        }

        write!(f, "{}", s)
    }
}

// I marked methods that might be performance-critical with #[inline]. I'm not sure it makes a
// difference, though.
impl Board {
    /// Creates a new `Board` from an array of human-looking numbers.
    pub fn new(grid: &[[u32; 4]; 4]) -> Option<Board> {
        let mut result = [[0; 4]; 4];

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

        Some(Board { grid: result })
    }

    /// Gets a reference to the inner representation of the `Board`, which is a 4x4 array of `u8`.
    #[inline]
    pub fn get_grid(&self) -> &[[u8; 4]; 4] {
        &self.grid
    }

    /// Gets a transposed copy of the inner representation of the `Board`.
    #[inline]
    pub fn transpose(&self) -> [[u8; 4]; 4] {
        let mut t = [[0; 4]; 4];

        let mut x = 0;
        let mut y = 0;
        for row in &self.grid {
            for val in row {
                t[y][x] = *val;
                x += 1;
            }

            y += 1;
        }

        t
    }

    /// Gets a reference to the inner representation of the `Board` as a flat array of `u8`.
    #[inline]
    pub fn flatten(&self) -> &[u8; 16] {
        use std::mem;
        // Not sure this is worth it to avoid copying 8 to 12 extra bytes of memory...
        unsafe { mem::transmute(&self.grid) }
    }

    /// Adds a random tile (10% of times a `2`, 90% of times a `4`) to a random empty cell on the
    /// board.
    pub fn add_random_tile(&self) -> Board {
        use std::mem;
        use rand::{self, Rng};

        let mut rng = rand::thread_rng();

        let mut flat = *self.flatten();

        let empty_cell_count = flat.iter().filter(|&&v| v == 0).count();

        let create_four = rng.gen_weighted_bool(10);
        let value = if create_four {
            2
        } else {
            1
        };
        
        let mut position = rng.gen_range(0, empty_cell_count);

        for val in &mut flat {
            if *val != 0 {
                continue;
            }

            if position == 0 {
                *val = value;
                break;
            }

            position -= 1;
        }

        // This unsafe block is certainly unnecessary, since this method is not
        // performance-critical, since it only happens once per whole search...
        // but I'm already using unsafe for doing it the other way.
        Board { grid: unsafe { mem::transmute(flat) } }
    }

    /// Returns a `Board` that would result from making a certain `Move` in the current state.
    #[inline]
    pub fn make_move(&self, mv: Move) -> Board {
        match mv {
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::Up => self.move_up(),
            Move::Down => self.move_down(),
        }
    }

    /// Returns all possible `Board`s that can result from the computer spawning a `2` in a random
    /// empty cell.
    #[inline]
    pub fn get_possible_boards_with2(&self) -> Vec<Board> {
        self.get_possible_boards(1)
    }

    /// Returns all possible `Board`s that can result from the computer spawning a `4` in a random
    /// empty cell.
    #[inline]
    pub fn get_possible_boards_with4(&self) -> Vec<Board> {
        self.get_possible_boards(2)
    }

    #[inline]
    fn get_possible_boards(&self, new_value: u8) -> Vec<Board> {
        let mut result = Vec::<Board>::new();

        for x in 0..4 {
            for y in 0..4 {
                if self.grid[x][y] != 0 {
                    continue;
                }

                let mut possible_grid = self.grid;
                possible_grid[x][y] = new_value;
                result.push(Board { grid: possible_grid });
            }
        }

        result
    }

    #[inline]
    fn move_left(&self) -> Board {
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
                    last_index += 1;
                    continue;
                }

                result[x][last_index] = last;
                last = current;
                last_index += 1;
            }

            if last != 0 {
                result[x][last_index] = last;
            }
        }

        Board { grid: result }
    }

    #[inline]
    fn move_right(&self) -> Board {
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
                    last_index -= 1;
                    continue;
                }

                result[x][last_index] = last;
                last = current;
                last_index -= 1;
            }

            if last != 0 {
                result[x][last_index] = last;
            }
        }

        Board { grid: result }
    }

    #[inline]
    fn move_up(&self) -> Board {
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
                    last_index += 1;
                    continue;
                }

                result[last_index][y] = last;
                last = current;
                last_index += 1;
            }

            if last != 0 {
                result[last_index][y] = last;
            }
        }

        Board { grid: result }
    }

    #[inline]
    fn move_down(&self) -> Board {
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
                    last_index -= 1;
                    continue;
                }

                result[last_index][y] = last;
                last = current;
                last_index -= 1;
            }

            if last != 0 {
                result[last_index][y] = last;
            }
        }

        Board { grid: result }
    }
}

fn get_human(n: u8) -> u32 {
    match n {
        0 => 0,
        _ => 1 << n,
    }
}

fn parse_to_logspace(n: u32) -> Option<u8> {
    use std::f64;

    let log = match n {
        0 => 0f64,
        _ => (n as f64).log2(),
    };

    let rounded = log.round();
    if (rounded - log) < 1e-10 {
        Some(rounded as u8)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_create_empty_board() {
        let expected = Board {
            grid: [
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
            ]
        };

        let actual = Board::default();

        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_create_board_from_human_input() {
        let expected: [[u8; 4]; 4] = [
            [0, 1, 2, 3],
            [4, 5, 6, 7],
            [8, 9, 10, 11],
            [12, 13, 14, 15]
        ];

        let actual = Board::new(&[
            [0, 2, 4, 8],
            [16, 32, 64, 128],
            [256, 512, 1024, 2048],
            [4096, 8192, 16384, 32768]
        ]);

        assert!(actual.is_some());
        assert_eq!(&expected, actual.unwrap().get_grid());
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_return_none_on_invalid_input() {
        let result = Board::new(&[
            [0, 1, 2, 3],
            [4, 5, 6, 7],
            [8, 9, 10, 11],
            [12, 13, 14, 15]
        ]);

        assert!(result.is_none());
    }

    #[test]
    fn can_add_random_tile() {
        let board = Board::default().add_random_tile();

        let count = board.flatten()
            .iter()
            .filter(|&&v| v == 1 || v == 2)
            .count();

        assert_eq!(1, count);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_to_string() {
        // arrange
        let board = Board::new(&[
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
        let actual = board.to_string();

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_make_move_left() {
        // arrange
        let board = Board::new(&[
            [2, 2, 4, 4],
            [0, 2, 2, 0],
            [0, 2, 2, 2],
            [2, 0, 0, 2]
        ]).unwrap();
        let expected = Board::new(&[
            [4, 8, 0, 0],
            [4, 0, 0, 0],
            [4, 2, 0, 0],
            [4, 0, 0, 0]
        ]).unwrap();

        // act
        let actual = board.make_move(Move::Left);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_make_move_right() {
        // arrange
        let board = Board::new(&[
            [2, 2, 4, 4],
            [0, 2, 2, 0],
            [0, 2, 2, 2],
            [2, 0, 0, 2]
        ]).unwrap();
        let expected = Board::new(&[
            [0, 0, 4, 8],
            [0, 0, 0, 4],
            [0, 0, 2, 4],
            [0, 0, 0, 4]
        ]).unwrap();

        // act
        let actual = board.make_move(Move::Right);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_make_move_up() {
        // arrange
        let board = Board::new(&[
            [2, 2, 4, 4],
            [0, 2, 2, 0],
            [0, 2, 2, 2],
            [2, 0, 0, 2]
        ]).unwrap();
        let expected = Board::new(&[
            [4, 4, 4, 4],
            [0, 2, 4, 4],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]).unwrap();

        // act
        let actual = board.make_move(Move::Up);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_make_move_down() {
        // arrange
        let board = Board::new(&[
            [2, 2, 4, 4],
            [0, 2, 2, 0],
            [0, 2, 2, 2],
            [2, 0, 0, 2]
        ]).unwrap();
        let expected = Board::new(&[
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 2, 4, 4],
            [4, 4, 4, 4]
        ]).unwrap();

        // act
        let actual = board.make_move(Move::Down);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_get_possible_boards_with2() {
        // arrange
        let board = Board::new(&[
            [0, 8, 8, 8],
            [8, 8, 0, 8],
            [8, 8, 8, 0],
            [8, 0, 8, 8]
        ]).unwrap();

        let expected = vec![
        Board::new(&[
            [2, 8, 8, 8],
            [8, 8, 0, 8],
            [8, 8, 8, 0],
            [8, 0, 8, 8]
        ]).unwrap(),
        Board::new(&[
            [0, 8, 8, 8],
            [8, 8, 2, 8],
            [8, 8, 8, 0],
            [8, 0, 8, 8]
        ]).unwrap(),
        Board::new(&[
            [0, 8, 8, 8],
            [8, 8, 0, 8],
            [8, 8, 8, 2],
            [8, 0, 8, 8]
        ]).unwrap(),
        Board::new(&[
            [0, 8, 8, 8],
            [8, 8, 0, 8],
            [8, 8, 8, 0],
            [8, 2, 8, 8]
        ]).unwrap()];

        // act
        let actual = board.get_possible_boards_with2();

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn can_get_possible_boards_with4() {
        // arrange
        let board = Board::new(&[
            [0, 8, 8, 8],
            [8, 8, 0, 8],
            [8, 8, 8, 0],
            [8, 0, 8, 8]
        ]).unwrap();

        let expected = vec![
        Board::new(&[
            [4, 8, 8, 8],
            [8, 8, 0, 8],
            [8, 8, 8, 0],
            [8, 0, 8, 8]
        ]).unwrap(),
        Board::new(&[
            [0, 8, 8, 8],
            [8, 8, 4, 8],
            [8, 8, 8, 0],
            [8, 0, 8, 8]
        ]).unwrap(),
        Board::new(&[
            [0, 8, 8, 8],
            [8, 8, 0, 8],
            [8, 8, 8, 4],
            [8, 0, 8, 8]
        ]).unwrap(),
        Board::new(&[
            [0, 8, 8, 8],
            [8, 8, 0, 8],
            [8, 8, 8, 0],
            [8, 4, 8, 8]
        ]).unwrap()];

        // act
        let actual = board.get_possible_boards_with4();

        // assert
        assert_eq!(expected, actual);
    }
}
