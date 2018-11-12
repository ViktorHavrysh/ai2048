//! `Board` represents the board state in a 2048 game.

use lazy_static::lazy_static;
use rand::{self, Rng};
use std::{fmt, u16};

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

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Default)]
pub(crate) struct Row(pub(crate) u16);

impl Row {
    pub(crate) fn pack(row: [u8; 4]) -> Option<Row> {
        let mut result = 0;
        for &cell in &row {
            if cell > 0b1111 {
                return None;
            }
            result <<= 4;
            result += cell as u16;
        }
        Some(Row(result))
    }

    pub(crate) fn unpack(self) -> [u8; 4] {
        let row = self.0;
        let col0 = ((row & 0b1111_0000_0000_0000) >> 12) as u8;
        let col1 = ((row & 0b0000_1111_0000_0000) >> 8) as u8;
        let col2 = ((row & 0b0000_0000_1111_0000) >> 4) as u8;
        let col3 = (row & 0b0000_0000_0000_1111) as u8;
        [col0, col1, col2, col3]
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
    pub(crate) rows: [Row; 4],
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.unpack_u32().iter() {
            for &cell in row {
                write!(f, "{number:>width$}", number = cell, width = 6)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl Board {
    /// Creates a new `Board` from an array of human-looking numbers. If a tile fails to be
    /// a power of 2, returns `None`.
    pub fn from_u32(grid: [[u32; 4]; 4]) -> Option<Board> {
        let mut board = Board::default();
        for (x, &row) in grid.iter().enumerate() {
            let mut new_row = [0u8; 4];
            for (y, &cell) in row.iter().enumerate() {
                let log = parse_to_log_space(cell)?;
                new_row[y] = log;
            }

            board.rows[x] = Row::pack(new_row)?;
        }
        Some(board)
    }

    fn from_u8(grid: [[u8; 4]; 4]) -> Option<Board> {
        let mut board = Board::default();
        for (x, &row) in grid.iter().enumerate() {
            board.rows[x] = Row::pack(row)?;
        }
        Some(board)
    }

    /// Unpacks a logarithmic representation from `Board`'s internal representation
    pub fn unpack_u8(self) -> [[u8; 4]; 4] {
        let mut result = [[0; 4]; 4];
        for (x, row) in self.rows.iter().enumerate() {
            result[x] = row.unpack();
        }
        result
    }

    /// Unpacks a human-readable representation from `Board`'s internal representation
    fn unpack_u32(self) -> [[u32; 4]; 4] {
        let mut result = [[0; 4]; 4];
        let board_u8 = self.unpack_u8();
        for (x, row) in board_u8.iter().enumerate() {
            for (y, &cell) in row.iter().enumerate() {
                result[x][y] = human(cell);
            }
        }
        result
    }

    /// Gets a transposed copy of the `Board`.
    #[inline]
    pub fn transpose(&self) -> Board {
        let row0 = self.rows[0].0;
        let row1 = self.rows[1].0;
        let row2 = self.rows[2].0;
        let row3 = self.rows[3].0;

        let row0_trans = (row0 & 0b1111_0000_0000_0000)
            + ((row1 & 0b1111_0000_0000_0000) >> 4)
            + ((row2 & 0b1111_0000_0000_0000) >> 8)
            + ((row3 & 0b1111_0000_0000_0000) >> 12);

        let row1_trans = ((row0 & 0b0000_1111_0000_0000) << 4)
            + (row1 & 0b0000_1111_0000_0000)
            + ((row2 & 0b0000_1111_0000_0000) >> 4)
            + ((row3 & 0b0000_1111_0000_0000) >> 8);

        let row2_trans = ((row0 & 0b0000_0000_1111_0000) << 8)
            + ((row1 & 0b0000_0000_1111_0000) << 4)
            + (row2 & 0b0000_0000_1111_0000)
            + ((row3 & 0b0000_0000_1111_0000) >> 4);

        let row3_trans = ((row0 & 0b0000_0000_0000_1111) << 12)
            + ((row1 & 0b0000_0000_0000_1111) << 8)
            + ((row2 & 0b0000_0000_0000_1111) << 4)
            + (row3 & 0b0000_0000_0000_1111);

        Board {
            rows: [
                Row(row0_trans),
                Row(row1_trans),
                Row(row2_trans),
                Row(row3_trans),
            ],
        }
    }

    /// Creates a new `Board` with a random tile (10% of times a `2`, 90% of times a `4`) added to a
    /// random empty cell on the board.
    pub fn add_random_tile(&self) -> Board {
        let mut rng = rand::thread_rng();

        let mut board = self.unpack_u32();
        let empty_cell_count = board.iter().flatten().filter(|v| **v == 0).count();
        let position = rng.gen_range(0, empty_cell_count);
        let create_four = rng.gen_bool(0.1);
        let value = if create_four { 4 } else { 2 };

        let val = board
            .iter_mut()
            .flatten()
            .filter(|v| **v == 0)
            .nth(position)
            .unwrap();

        *val = value;

        Board::from_u32(board).unwrap()
    }

    /// Returns all possible `Board`s that can result from the computer spawning a `2` in a random
    /// empty cell.
    pub fn ai_moves_with2(&self) -> Vec<Board> {
        self.ai_moves(1)
    }

    /// Returns all possible `Board`s that can result from the computer spawning a `4` in a random
    /// empty cell.
    pub fn ai_moves_with4(&self) -> Vec<Board> {
        self.ai_moves(2)
    }

    fn ai_moves(&self, new_value: u8) -> Vec<Board> {
        let board = self.unpack_u8();
        let mut boards = Vec::new();

        for (x, y) in board.iter().enumerate().flat_map(|(x, row)| {
            row.iter()
                .enumerate()
                .filter(|&(_, val)| *val == 0)
                .map(move |(y, _)| (x, y))
        }) {
            let mut new_board = board;
            new_board[x][y] = new_value;
            boards.push(Board::from_u8(new_board).unwrap());
        }

        boards
    }

    /// Returns a `Board` that would result from making a certain `Move` in the current state.
    pub fn make_move(&self, mv: Move) -> Board {
        match mv {
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::Up => self.transpose().move_left().transpose(),
            Move::Down => self.transpose().move_right().transpose(),
        }
    }

    fn move_left(&self) -> Board {
        let mut board = Board::default();

        for (to_row, from_row) in board.rows.iter_mut().zip(self.rows.iter()) {
            *to_row = Self::move_row_left_cached(*from_row);
        }

        board
    }

    fn move_right(&self) -> Board {
        let mut board = Board::default();

        for (to_row, from_row) in board.rows.iter_mut().zip(self.rows.iter()) {
            *to_row = Self::move_row_right_cached(*from_row)
        }

        board
    }

    #[inline]
    fn move_row_left_cached(row: Row) -> Row {
        CACHE_LEFT[row.0 as usize]
    }

    #[inline]
    fn move_row_right_cached(row: Row) -> Row {
        CACHE_RIGHT[row.0 as usize]
    }
}

fn move_row_left(row: Row) -> Row {
    let from_row = row.unpack();

    let mut to_row = [0; 4];
    let mut last = 0;
    let mut last_index = 0;

    for y in 0..4 {
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

        last_index += 1;
    }

    if last != 0 {
        to_row[last_index as usize] = last;
    }

    Row::pack(to_row).unwrap_or(Row::default())
}

fn move_row_right(row: Row) -> Row {
    let from_row = row.unpack();

    let mut to_row = [0; 4];
    let mut last = 0;
    let mut last_index = 3;

    for y in (0..4).rev() {
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

        last_index += -1;
    }

    if last != 0 {
        to_row[last_index as usize] = last;
    }

    Row::pack(to_row).unwrap_or(Row::default())
}

lazy_static! {
    static ref CACHE_LEFT: [Row; u16::MAX as usize] = {
        let mut cache = [Row::default(); u16::MAX as usize];
        for (index, row) in cache.iter_mut().enumerate() {
            *row = move_row_left(Row(index as u16));
        }

        cache
    };
    static ref CACHE_RIGHT: [Row; u16::MAX as usize] = {
        let mut cache = [Row::default(); u16::MAX as usize];
        for (index, row) in cache.iter_mut().enumerate() {
            *row = move_row_right(Row(index as u16));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_empty_board() {
        let expected =
            Board::from_u32([[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]).unwrap();

        let actual = Board::default();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_create_board_from_human_input() {
        let expected: [[u8; 4]; 4] = [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]];

        let actual = Board::from_u32([
            [0, 2, 4, 8],
            [16, 32, 64, 128],
            [256, 512, 1024, 2048],
            [4096, 8192, 16384, 32768],
        ]);

        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap().unpack_u8());
    }

    #[test]
    fn can_return_none_on_invalid_input() {
        let result =
            Board::from_u32([[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]);

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
                .unpack_u8()
                .iter()
                .flatten()
                .filter(|&&v| v == 1 || v == 2)
                .count();

            assert_eq!(8, count);
        }
    }

    #[test]
    fn can_to_string() {
        let board = Board::from_u32([
            [0, 2, 4, 8],
            [16, 32, 64, 128],
            [256, 512, 1024, 2048],
            [4096, 8192, 16384, 32768],
        ])
        .unwrap();

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
        let board =
            Board::from_u32([[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
        let expected =
            Board::from_u32([[4, 8, 0, 0], [4, 0, 0, 0], [4, 2, 0, 0], [4, 0, 0, 0]]).unwrap();

        let actual = board.make_move(Move::Left);

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_right() {
        let board =
            Board::from_u32([[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
        let expected =
            Board::from_u32([[0, 0, 4, 8], [0, 0, 0, 4], [0, 0, 2, 4], [0, 0, 0, 4]]).unwrap();

        let actual = board.make_move(Move::Right);

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_up() {
        let board =
            Board::from_u32([[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
        let expected =
            Board::from_u32([[4, 4, 4, 4], [0, 2, 4, 4], [0, 0, 0, 0], [0, 0, 0, 0]]).unwrap();

        let actual = board.make_move(Move::Up);

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_make_move_down() {
        let board =
            Board::from_u32([[2, 2, 4, 4], [0, 2, 2, 0], [0, 2, 2, 2], [2, 0, 0, 2]]).unwrap();
        let expected =
            Board::from_u32([[0, 0, 0, 0], [0, 0, 0, 0], [0, 2, 4, 4], [4, 4, 4, 4]]).unwrap();

        let actual = board.make_move(Move::Down);

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_possible_boards_with2() {
        let board =
            Board::from_u32([[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap();

        let expected = vec![
            Board::from_u32([[2, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap(),
            Board::from_u32([[0, 8, 8, 8], [8, 8, 2, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap(),
            Board::from_u32([[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 2], [8, 0, 8, 8]]).unwrap(),
            Board::from_u32([[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 2, 8, 8]]).unwrap(),
        ];

        let actual = board.ai_moves_with2();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_possible_boards_with4() {
        let board =
            Board::from_u32([[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap();

        let expected = vec![
            Board::from_u32([[4, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap(),
            Board::from_u32([[0, 8, 8, 8], [8, 8, 4, 8], [8, 8, 8, 0], [8, 0, 8, 8]]).unwrap(),
            Board::from_u32([[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 4], [8, 0, 8, 8]]).unwrap(),
            Board::from_u32([[0, 8, 8, 8], [8, 8, 0, 8], [8, 8, 8, 0], [8, 4, 8, 8]]).unwrap(),
        ];

        let actual = board.ai_moves_with4();

        assert_eq!(expected, actual);
    }
}
