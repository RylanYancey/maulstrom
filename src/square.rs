
//! Struct for representing squares on a chess board.

use std::{cmp::Ordering, mem, ops::{Add, BitOr}};
use crate::board::BitBoard;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Square {
    pub rank: i8,
    pub file: i8,
}

impl Square {
    pub const ZERO: Self = Self { rank: 0, file: 0 };

    pub const fn new(rank: i8, file: i8) -> Self {
        Self { rank, file }
    }

    pub const fn to_mask(&self) -> u64 {
        1 << self.to_index() as u64
    }

    pub const fn to_index(&self) -> usize {
        (self.file | self.rank << 3) as usize
    }

    pub const fn from_index(i: usize) -> Self {
        Self {
            rank: (i >> 3) as i8,
            file: (i & 7) as i8,
        }
    }

    pub const fn shares_orthogonal(&self, rhs: Self) -> bool {
        self.rank == rhs.rank || self.file == rhs.file
    }

    pub const fn shares_diagonal(&self, rhs: Self) -> bool {
        (self.rank - rhs.rank).abs() == (self.file - rhs.file).abs()
    }

    /// Get a mask of squares between self and rhs.
    pub const fn between(&self, rhs: Self) -> BitBoard {
        let mut i = self.file | (self.rank << 3);
        let mut j = rhs.file | (rhs.rank << 3);
        if i == j { return BitBoard(0) }
        if i > j { mem::swap(&mut i, &mut j) }

        let shared = if self.rank == rhs.rank {
            0xFF << (self.rank << 3)
        } else if self.file == rhs.file {
            0x0101010101010101 << self.file
        } else if self.rank - self.file == rhs.rank - rhs.file {
            let s = self.rank - self.file;
            if s > 0 { 0x8040201008040201 >> s }
            else { 0x8040201008040201 << s.abs() }
        } else if self.rank + self.file == rhs.rank + rhs.file {
            let s = self.rank - (7 - self.file);
            if s > 0 { 0x102040810204080 << s } 
            else { 0x102040810204080 >> s.abs() }
        } else {
            return BitBoard::new()
        };

        let a = !0u64 << (i + 1);
        let b = !0u64 >> (64 - j);
        BitBoard(shared & a & b)
    }

    /// Get a mask of the squares that share a diagonal with this square, inclusively.
    pub const fn diagonals(&self) -> BitBoard {
        const RGT: u64 = 0x8040201008040201;
        const LFT: u64 = 0x102040810204080;
        let rs = self.rank - self.file;
        let ra = rs.abs();
        let mut ri = ra << 3;
        if rs < 0 { ri = 64 - ri }
        let mut rm = (1 << ri) - 1;
        if rs > 0 { rm = !rm }
        let mut rg = RGT;
        if rs > 0 { rg = (rg >> ra) & rm }
        if rs < 0 { rg = (rg << ra) & rm }
        let ls = self.rank - (7 - self.file);
        let la = ls.abs();
        let mut li = la << 3;
        if ls < 0 { li = 63 - li }
        if ls > 0 { li += 1}
        let mut lm = (1 << li) - 1;
        if ls > 0 { lm = !lm }
        let mut lg = LFT;
        if ls > 0 { lg = (lg << la) & lm }
        if ls < 0 { lg = (lg >> la) & lm }
        BitBoard(rg | lg)
    }

    /// Get a mask of the squares that share an orthogonal with this square, inclusively.
    pub const fn orthogonals(&self) -> BitBoard {
        BitBoard((0xFF << (self.rank << 3)) | (0x0101010101010101 << self.file))
    }

    /// Check whether the square is within the bounds of a 
    /// standard 8x8 chess board.
    pub const fn is_in_bounds(&self) -> bool {
        (self.rank as u8 | self.file as u8) < 8
    }
}

impl From<(i8, i8)> for Square {
    fn from(value: (i8, i8)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl Add for Square {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            rank: self.rank + rhs.rank,
            file: self.file + rhs.file,
        }
    }
}

impl Add<(i8, i8)> for Square {
    type Output = Self;

    fn add(self, rhs: (i8, i8)) -> Self::Output {
        Self {
            rank: self.rank + rhs.0,
            file: self.file + rhs.1,
        }
    }
}

impl BitOr<Square> for Square {
    type Output = BitBoard;

    fn bitor(self, rhs: Square) -> Self::Output {
        BitBoard::from(self) | rhs
    }
}

impl PartialOrd for Square {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.rank != other.rank { self.rank.partial_cmp(&other.rank) }
        else { self.file.partial_cmp(&other.file) }
    }
}

impl Ord for Square {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

