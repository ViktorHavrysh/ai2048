//! `Board` represents the board state in a 2048 game.

use integer_magic::{u16_to_u8x4, u8x4_to_u16};
use rand::{self, Rng};
use std::{fmt, u16};

/// `Board` saves its state as a 4x4 array of `u8` values.
///
/// To cram the value of a cell into into one byte of memory, `Board` uses a logarithmic
/// representation of the value displayed to the player. That is, `2` becomes `1`,
/// `4` becomes `2`, `8` becomes `3`, etc. The maximum cell value theoretically achievable in a
/// standard game of 2048 is `65,536`, and that is represented by the value `16`, so a byte is
/// more than enough storage for a single cell. `0` stays a `0`.
///
/// `Board`, in general, encodes all the rules of the game: it can generate new states
/// given a move a player makes, or all possible states following the computer spawning a random
/// tile. Unsurprisingly, in order to write an AI for a game, the AI needs an emulation of the
/// game itself.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Default)]
pub struct Board {
    grid: [[u8; 4]; 4],
}

/// Represents a move.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Move {
    /// Move left.
    Left = 0,
    /// Move right.
    Right = 1,
    /// Move up.
    Up = 2,
    /// Move down.
    Down = 3,
}

/// All possible moves.
pub const MOVES: [Move; 4] = [Move::Left, Move::Right, Move::Up, Move::Down];

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.grid {
            for &val in row {
                let human = human(val);
                write!(f, "{number:>width$}", number = human, width = 6)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let move_str = match *self {
            Move::Down => "Down",
            Move::Left => "Left",
            Move::Right => "Right",
            Move::Up => "Up",
        };
        move_str.fmt(f)
    }
}

// I marked methods that might be performance-critical with #[inline]. I'm not sure it makes a
// difference, though.
impl Board {
    /// Creates a new `Board` from an array of human-looking numbers. If a tile fails to be
    /// a power of 2, returns `None`.
    pub fn new(grid: &[[u32; 4]; 4]) -> Option<Board> {
        let mut result = [[0; 4]; 4];

        for (x, row) in grid.iter().enumerate() {
            for (y, &val) in row.iter().enumerate() {
                let log = parse_to_log_space(val);

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

    /// Gets the maximum number of moves needed to get to this game position
    pub fn moves(&self) -> u32 {
        self.grid
            .iter()
            .flatten()
            .map(|&cell| u32::from(cell))
            .sum()
    }

    /// Gets a reference to the inner representation of the `Board`, which is a 4x4 array of `u8`.
    #[inline]
    pub fn grid(&self) -> &[[u8; 4]; 4] {
        &self.grid
    }

    /// Gets a transposed copy of the `Board`.
    #[inline]
    pub fn transpose(&self) -> Board {
        let row0 = [
            self.grid[0][0],
            self.grid[1][0],
            self.grid[2][0],
            self.grid[3][0],
        ];
        let row1 = [
            self.grid[0][1],
            self.grid[1][1],
            self.grid[2][1],
            self.grid[3][1],
        ];
        let row2 = [
            self.grid[0][2],
            self.grid[1][2],
            self.grid[2][2],
            self.grid[3][2],
        ];
        let row3 = [
            self.grid[0][3],
            self.grid[1][3],
            self.grid[2][3],
            self.grid[3][3],
        ];

        let grid = [row0, row1, row2, row3];

        Board { grid }
    }

    /// Creates a new `Board` with a random tile (10% of times a `2`, 90% of times a `4`) added to a
    /// random empty cell on the board.
    pub fn add_random_tile(&self) -> Board {
        let mut rng = rand::thread_rng();
        let empty_cell_count = self.grid.iter().flatten().filter(|v| **v == 0).count();
        let position = rng.gen_range(0, empty_cell_count);
        let create_four = rng.gen_weighted_bool(10);
        let value = if create_four { 2 } else { 1 };

        let mut new_grid = self.grid;

        {
            let val = new_grid
                .iter_mut()
                .flatten()
                .filter(|v| **v == 0)
                .nth(position)
                .unwrap();

            *val = value;
        }

        Board { grid: new_grid }
    }

    /// Returns all possible `Board`s that can result from the computer spawning a `2` in a random
    /// empty cell.
    #[inline]
    pub fn possible_boards_with2<'a>(&'a self) -> impl Iterator<Item = Board> + 'a {
        self.possible_boards(1)
    }

    /// Returns all possible `Board`s that can result from the computer spawning a `4` in a random
    /// empty cell.
    #[inline]
    pub fn possible_boards_with4<'a>(&'a self) -> impl Iterator<Item = Board> + 'a {
        self.possible_boards(2)
    }

    #[inline]
    fn possible_boards<'a>(&'a self, new_value: u8) -> impl Iterator<Item = Board> + 'a {
        self.grid
            .into_iter()
            .enumerate()
            .flat_map(|(x, row)| {
                row.into_iter()
                    .enumerate()
                    .filter(|&(_, val)| *val == 0)
                    .map(move |(y, _)| (x, y))
            })
            .map(move |(x, y)| {
                let mut possible_grid = self.grid;
                possible_grid[x][y] = new_value;
                Board {
                    grid: possible_grid,
                }
            })
    }

    /// Returns a `Board` that would result from making a certain `Move` in the current state.
    #[inline]
    pub fn make_move(&self, mv: Move) -> Board {
        match mv {
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::Up => self.transpose().move_left().transpose(),
            Move::Down => self.transpose().move_right().transpose(),
        }
    }

    #[inline]
    fn move_left(&self) -> Board {
        let mut result = [[0; 4]; 4];

        for (to_row, from_row) in result.iter_mut().zip(self.grid.iter()) {
            *to_row = Self::move_row_left_cached(*from_row);
        }

        Board { grid: result }
    }

    #[inline]
    fn move_right(&self) -> Board {
        let mut result = [[0; 4]; 4];

        for (to_row, from_row) in result.iter_mut().zip(self.grid.iter()) {
            *to_row = Self::move_row_right_cached(*from_row)
        }

        Board { grid: result }
    }

    #[inline]
    fn move_row_left_cached(row: [u8; 4]) -> [u8; 4] {
        match u8x4_to_u16(row) {
            Some(u) => CACHE_LEFT[u as usize],
            None => Self::move_row_left(row),
        }
    }

    #[inline]
    fn move_row_right_cached(row: [u8; 4]) -> [u8; 4] {
        match u8x4_to_u16(row) {
            Some(u) => CACHE_RIGHT[u as usize],
            None => Self::move_row_right(row),
        }
    }

    #[inline]
    fn move_row_left(row: [u8; 4]) -> [u8; 4] {
        Self::move_row(&row, 0..4, 1, 0)
    }

    #[inline]
    fn move_row_right(row: [u8; 4]) -> [u8; 4] {
        Self::move_row(&row, (0..4).rev(), -1, 3)
    }

    #[inline]
    fn move_row<I>(from_row: &[u8; 4], iter: I, step: isize, mut last_index: isize) -> [u8; 4]
    where
        I: Iterator<Item = usize>,
    {
        let mut to_row = [0; 4];
        let mut last = 0;

        for y in iter {
            let current = from_row[y];

            if current == 0 {
                continue;
            }

            if last == 0 {
                last = current;
                continue;
            }

            if current == last {
                to_row[last_index as usize] = last + 1;
                last = 0;
            } else {
                to_row[last_index as usize] = last;
                last = current;
            }

            last_index += step;
        }

        if last != 0 {
            to_row[last_index as usize] = last;
        }

        to_row
    }
}

lazy_static! {
    static ref CACHE_LEFT: [[u8; 4]; u16::MAX as usize] = {
        let mut cache = [[0; 4]; u16::MAX as usize];
        for (index, mut row) in cache.iter_mut().enumerate() {
            *row = Board::move_row_left(u16_to_u8x4(index as u16));
        }

        cache
    };

    static ref CACHE_RIGHT: [[u8; 4]; u16::MAX as usize] = {
        let mut cache = [[0; 4]; u16::MAX as usize];
        for (index, mut row) in cache.iter_mut().enumerate() {
            *row = Board::move_row_right(u16_to_u8x4(index as u16));
        }

        cache
    };
}

fn human(n: u8) -> u32 {
    match n {
        0 => 0,
        _ => 1 << n,
    }
}

fn parse_to_log_space(n: u32) -> Option<u8> {
    use std::f32;

    let log = match n {
        0 => 0f32,
        _ => (n as f32).log2(),
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
    use itertools::Itertools;

    #[test]
    fn can_create_empty_board() {
        let expected = Board {
            grid: [[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
        };

        let actual = Board::default();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_create_board_from_human_input() {
        let expected: [[u8; 4]; 4] = [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]];

        let actual = Board::new(&[
            [0, 2, 4, 8],
            [16, 32, 64, 128],
            [256, 512, 1024, 2048],
            [4096, 8192, 16384, 32768],
        ]);

        assert!(actual.is_some());
        assert_eq!(&expected, actual.unwrap().grid());
    }

    #[test]
    fn can_return_none_on_invalid_input() {
        let result = Board::new(&[[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]);

        assert!(result.is_none());
    }

    #[test]
    fn can_add_random_tile() {
        for _ in 0..1000 {
            let mut board = Board::default();
            for _ in 0..8 {
                board = board.add_random_tile();
            }

            let count = board
                .grid()
                .iter()
                .flatten()
                .filter(|&&v| v == 1 || v == 2)
                .count();

            assert_eq!(8, count);
        }
    }

    #[test]
    fn can_to_string() {
        let board = Board::new(&[
            [0, 2, 4, 8],
            [16, 32, 64, 128],
            [256, 512, 1024, 2048],
            [4096, 8192, 16384, 32768],
        ]).unwrap();

        let mut expected = String::new();
        expected.push_str("     0     2     4     8\n");
        expected.push_str("    16    32    64   128\n");
        expected.push_str("   256   512  1024  2048\n");
        expected.push_str("  4096  8192 16384 32768\n");

        let actual = board.to_string();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_left() {
        let board = Board::new(&[[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
        let expected =
            Board::new(&[[4, 8, 0, 0], [4, 0, 0, 0], [4, 2, 0, 0], [4, 0, 0, 0]]).unwrap();

        let actual = board.make_move(Move::Left);

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_right() {
        let board = Board::new(&[[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
        let expected =
            Board::new(&[[0, 0, 4, 8], [0, 0, 0, 4], [0, 0, 2, 4], [0, 0, 0, 4]]).unwrap();

        let actual = board.make_move(Move::Right);

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_up() {
        let board = Board::new(&[[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
        let expected =
            Board::new(&[[4, 4, 4, 4], [0, 2, 4, 4], [0, 0, 0, 0], [0, 0, 0, 0]]).unwrap();

        let actual = board.make_move(Move::Up);

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_down() {
        let board = Board::new(&[[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
        let expected =
            Board::new(&[[0, 0, 0, 0], [0, 0, 0, 0], [0, 2, 4, 4], [4, 4, 4, 4]]).unwrap();

        let actual = board.make_move(Move::Down);

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_possible_boards_with2() {
        let board = Board::new(&[[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap();

        let expected = vec![
            Board::new(&[[2, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap(),
            Board::new(&[[0, 8, 8, 8], [8, 8, 2, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap(),
            Board::new(&[[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 2], [8, 0, 8, 8]]).unwrap(),
            Board::new(&[[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 2, 8, 8]]).unwrap(),
        ];

        let actual = board.possible_boards_with2().collect::<Vec<_>>();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_possible_boards_with4() {
        let board = Board::new(&[[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap();

        let expected = vec![
            Board::new(&[[4, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap(),
            Board::new(&[[0, 8, 8, 8], [8, 8, 4, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap(),
            Board::new(&[[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 4], [8, 0, 8, 8]]).unwrap(),
            Board::new(&[[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 4, 8, 8]]).unwrap(),
        ];

        let actual = board.possible_boards_with4().collect::<Vec<_>>();

        assert_eq!(expected, actual);
    }
}
