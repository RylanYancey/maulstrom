
//! Structs for working with Bitboards.

use std::fmt::{self, Write};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::team::Team;

use super::square::Square;

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub const EMPTY: Self = Self(0);

    pub const fn new() -> Self {
        Self(0)
    }

    pub fn full() -> Self {
        Self(!0)
    }

    pub const fn set(&mut self, sq: Square) {
        self.0 |= sq.to_mask()
    }

    pub const fn with(self, sq: Square) -> Self {
        Self(self.0 | sq.to_mask())
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn count(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub const fn clear(&mut self, sq: Square) -> bool {
        let old = self.0;
        self.0 &= !sq.to_mask();
        old == self.0
    }

    pub const fn clear_all(&mut self, board: BitBoard) -> bool {
        let old = self.0;
        self.0 &= !board.0;
        old == self.0
    }

    /// Mask all bits before the index, exclusive.
    pub const fn before(i: usize) -> Self {
        Self(!0u64 >> (64 - i))
    }

    /// Mask all bits after the index, exclusive.
    pub const fn after(i: usize) -> Self {
        Self(!0u64 << (i + 1))
    }

    /// Whether the two masks have any shared bits.
    pub const fn intersects(&self, rhs: Self) -> bool {
        self.0 & rhs.0 != 0
    }

    pub const fn without(self, sq: Square) -> Self {
        Self(self.0 &! sq.to_mask())
    }

    pub const fn is_set(&self, sq: Square) -> bool {
        self.0 & sq.to_mask() != 0
    }

    /// same as is_set
    pub const fn has(&self, sq: Square) -> bool {
        self.0 & sq.to_mask() != 0 
    }

    pub const fn set_rank_u8(&mut self, rank: u8) {
        self.0 |= 0xFF << (rank * 8);
    }

    pub const fn with_rank_u8(self, rank: u8) -> Self {
        Self(self.0 | (0xFF << (rank * 8)))
    }

    pub const fn set_file_u8(&mut self, file: u8) {
        self.0 |= 0x0101010101010101 << file
    }

    pub const fn with_file_u8(&mut self, file: u8) -> Self {
        Self(self.0 | (0x0101010101010101 << file))
    }

    /// Get the first (lowest x,y) square in the mask.
    pub const fn first(&self) -> Option<Square> {
        if self.0 != 0 {
            Some(Square::from_index(self.0.trailing_zeros() as usize))
        } else {
            None
        }
    }

    /// Get the last (highest x,y) square in the mask.
    pub const fn last(&self) -> Option<Square> {
        if self.0 != 0 {
            Some(Square::from_index(63 - self.0.leading_zeros() as usize))
        } else {
            None
        }
    }

    /// Const version of the & operator.
    pub const fn and(&self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    pub const fn indices(&self) -> BitBoardIndices {
        BitBoardIndices(self.0)
    }

    pub const fn transmit(&self, tx: Self) -> Self {
        if self.intersects(tx) {
            Self(self.0 | tx.0)
        } else {
            *self
        }
    }

    pub const fn pawn_captures(&self, team: Team) -> BitBoard {
        let pr = self.0 & !0x8080808080808080;
        let pl = self.0 & !0x0101010101010101;
        match team {
            Team::White => BitBoard((pl << 7) | (pr << 9)),
            Team::Black => BitBoard((pl >> 9) | (pr >> 7))
        }
    }
}

impl From<u64> for BitBoard {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Square> for BitBoard {
    fn from(value: Square) -> Self {
        Self(1 << value.to_index())
    }
}

impl From<(u8, u8)> for BitBoard {
    fn from(value: (u8, u8)) -> Self {
        Self::from(Square::new(value.0.into(), value.1.into()))
    }
}

impl IntoIterator for BitBoard {
    type IntoIter = BitBoardIter;
    type Item = Square;

    fn into_iter(self) -> Self::IntoIter {
        BitBoardIter(self.0)
    }
}

#[derive(Copy, Clone)]
pub struct BitBoardIter(u64);

impl Iterator for BitBoardIter {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None
        }

        let i = self.0.trailing_zeros();
        self.0 &= !(1u64 << i);
        Some(Square::from_index(i as usize))
    }
}

impl<T: Into<BitBoard>> BitAnd<T> for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: T) -> Self::Output {
        Self(self.0 & rhs.into().0)
    }
}

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl<T: Into<BitBoard>> BitAndAssign<T> for BitBoard {
    fn bitand_assign(&mut self, rhs: T) {
        self.0 &= rhs.into().0
    }
}

impl<T: Into<BitBoard>> BitOrAssign<T> for BitBoard {
    fn bitor_assign(&mut self, rhs: T) {
        self.0 |= rhs.into().0
    }
}

impl<T: Into<BitBoard>> BitOr<T> for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: T) -> Self::Output {
        Self(self.0 | rhs.into().0)
    }
}

impl From<bool> for BitBoard {
    fn from(value: bool) -> Self {
        if value {
            Self(!0)
        } else {
            Self(0)
        }
    }
}

impl<T: Into<BitBoard>> BitXor<T> for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: T) -> Self::Output {
        Self(self.0 ^ rhs.into().0)
    }
}

impl<T: Into<BitBoard>> BitXorAssign<T> for BitBoard {
    fn bitxor_assign(&mut self, rhs: T) {
        self.0 ^= rhs.into().0
    }
}

impl fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BitBoard(\n")?;
        for x in (0..8).rev() {
            f.write_char('\t')?;
            for y in 0..8 {
                if self.has((x, y).into()) {
                    f.write_str("o ")?;
                } else {
                    f.write_str("- ")?;
                }
            }
            f.write_char('\n')?;
        }
        f.write_str(")\n")
    }
}

pub struct BitBoardIndices(pub u64);

impl Iterator for BitBoardIndices {
    type Item = u32;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let i = self.0.trailing_zeros();
        if i != 64 {
            self.0 ^= 1 << i;
            Some(i)
        } else {
            None
        }
    }
}