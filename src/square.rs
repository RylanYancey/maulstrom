
//! Struct for representing squares on a chess board.

use std::{cmp::Ordering, fmt, mem, ops::{Add, BitOr}};
use crate::{board::BitBoard, cached::*, ray::Ray, team::Team};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum Rank {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

impl Rank {
    pub const fn to_u8(&self) -> u8 {
        match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
            Self::Fourth => 3,
            Self::Fifth => 4,
            Self::Sixth => 5,
            Self::Seventh => 6,
            Self::Eighth => 7,
        }
    }
}

impl From<u8> for Rank {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::First,
            1 => Self::Second,
            2 => Self::Third,
            3 => Self::Fourth,
            4 => Self::Fifth,
            5 => Self::Sixth,
            6 => Self::Seventh,
            _ => Self::Eighth,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum File {
    A, B, C, D, E, F, G, H
}

impl File {
    pub const fn to_u8(&self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
            Self::E => 4,
            Self::F => 5,
            Self::G => 6,
            Self::H => 7, 
        }
    }

    pub fn from_i8(n: i8) -> Option<Self> {
        if n >= 0 && n < 8 {
            Some(Self::from(n as u8))
        } else {
            None
        }
    }
}

impl From<u8> for File {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            6 => Self::G,
            _ => Self::H
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Square(u8);

impl Square {
    pub const ZERO: Self = Self (0);

    pub const fn new(rank: Rank, file: File) -> Self {
        Self((rank.to_u8() << 3) | file.to_u8())
    }

    pub fn rank(&self) -> Rank {
        (self.0 >> 3).into()
    }

    pub fn file(&self) -> File {
        (self.0 & 0b111).into()
    }

    pub const fn rank_u8(&self) -> u8 {
        self.0 >> 3
    }

    pub const fn file_u8(&self) -> u8 {
        self.0 & 0b111
    }

    pub const fn to_mask(&self) -> u64 {
        1 << self.0 as u64
    }

    pub const fn to_index(&self) -> usize {
        self.0 as usize
    }

    pub const fn from_index(i: usize) -> Self {
        Self(i as u8)
    }

    /// Squares between self and rhs, including rhs and excluding self.
    pub const fn between(&self, rhs: Self) -> BitBoard {
        BitBoard(crate::cached::BETWEEN_EXCLUSIVE[self.to_index()][rhs.to_index()])
    }

    pub fn pawn_captures(&self, team: Team) -> BitBoard {
        match team {
            Team::White => BitBoard(crate::cached::WHITE_PAWN_ATTACKS[self.to_index()]),
            Team::Black => BitBoard(crate::cached::BLACK_PAWN_ATTACKS[self.to_index()])
        }
    }

    pub const fn next(&self, delta: (i8, i8)) -> Option<Self> {
        let rank = self.rank_u8() as i8 + delta.0;
        let file = self.file_u8() as i8 + delta.1;
        if rank >= 0 && rank < 8 && file >= 0 && file < 8 {
            Some(Self(((rank as u8) << 3) | file as u8))
        } else {
            None
        }
    }

    pub fn king_moves(&self) -> BitBoard {
        BitBoard(crate::cached::KING_MOVES[self.0 as usize])
    }

    pub fn knight_moves(&self) -> BitBoard {
        BitBoard(crate::cached::KNIGHT_MOVES[self.0 as usize])
    }

    pub fn rook_moves(&self, occupied: BitBoard) -> BitBoard {
        crate::magic::get_rook_moves(*self, occupied)
    }

    pub fn bishop_moves(&self, occupied: BitBoard) -> BitBoard {
        crate::magic::get_bishop_moves(*self, occupied)
    }

    pub const fn ortho_ray(&self, rhs: Self) -> Option<Ray> {
        let i = self.0 as usize;
        let j = rhs.to_mask();
        if RAY_POS_ZERO_EXCLUSIVE[i] & j != 0 { return Some(Ray::PosZero) }
        if RAY_NEG_ZERO_EXCLUSIVE[i] & j != 0 { return Some(Ray::NegZero) }
        if RAY_ZERO_NEG_EXCLUSIVE[i] & j != 0 { return Some(Ray::ZeroNeg) }
        if RAY_ZERO_POS_EXCLUSIVE[i] & j != 0 { return Some(Ray::ZeroPos) }
        None
    }

    pub const fn diag_ray(&self, rhs: Self) -> Option<Ray> {
        let i = self.0 as usize;
        let j = rhs.to_mask();
        if RAY_POS_POS_EXCLUSIVE[i] & j != 0 { return Some(Ray::PosPos); }
        if RAY_NEG_NEG_EXCLUSIVE[i] & j != 0 { return Some(Ray::NegNeg); }
        if RAY_NEG_POS_EXCLUSIVE[i] & j != 0 { return Some(Ray::NegPos); }
        if RAY_POS_NEG_EXCLUSIVE[i] & j != 0 { return Some(Ray::PosNeg); }
        None
    }

    pub const fn ray(&self, rhs: Self) -> Option<Ray> {
        let i = self.0 as usize;
        let j = rhs.to_mask();
        if RAY_POS_ZERO_EXCLUSIVE[i] & j != 0 { return Some(Ray::PosZero) }
        if RAY_NEG_ZERO_EXCLUSIVE[i] & j != 0 { return Some(Ray::NegZero) }
        if RAY_ZERO_NEG_EXCLUSIVE[i] & j != 0 { return Some(Ray::ZeroNeg) }
        if RAY_ZERO_POS_EXCLUSIVE[i] & j != 0 { return Some(Ray::ZeroPos) }
        if RAY_POS_POS_EXCLUSIVE[i] & j != 0 { return Some(Ray::PosPos); }
        if RAY_NEG_NEG_EXCLUSIVE[i] & j != 0 { return Some(Ray::NegNeg); }
        if RAY_NEG_POS_EXCLUSIVE[i] & j != 0 { return Some(Ray::NegPos); }
        if RAY_POS_NEG_EXCLUSIVE[i] & j != 0 { return Some(Ray::PosNeg); }
        None
    }
}

impl From<(u8, u8)> for Square {
    fn from(value: (u8, u8)) -> Self {
        Self::new(value.0.into(), value.1.into())
    }
}

impl From<(Rank, File)> for Square {
    fn from(value: (Rank, File)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl BitOr<Square> for Square {
    type Output = BitBoard;

    fn bitor(self, rhs: Square) -> Self::Output {
        BitBoard::from(self) | rhs
    }
}

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Square")
            .field("rank", &self.rank())
            .field("file", &self.file())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::BitBoard;

    #[test]
    fn king_moves() {
        const KING_DELTAS: [(i8, i8); 8] = [
            (-1, -1),
            (-1,  0),
            (-1,  1),
            ( 0, -1),
            ( 0,  1),
            ( 1, -1),
            ( 1,  0),
            ( 1,  1)
        ];

        for square in BitBoard(!0) {
            let mut expected = BitBoard::new();
            for delta in KING_DELTAS {
                if let Some(next) = square.next(delta) {
                    expected |= next;
                }
            }

            assert_eq!(square.king_moves(), expected, "{square:?}");
        }
    }

    #[test]
    fn knight_moves() {
        const KNIGHT_DELTAS: [(i8, i8); 8] = [
            (-1, -2),
            ( 1, -2),
            (-1,  2),
            ( 1,  2),
            (-2, -1),
            (-2,  1),
            ( 2, -1),
            ( 2,  1)
        ];
    
        for square in BitBoard(!0) {
            let mut expected = BitBoard::new();
            for delta in KNIGHT_DELTAS {
                if let Some(next) = square.next(delta) {
                    expected |= next;
                }
            }
    
            assert_eq!(square.knight_moves(), expected, "{square:?}");
        }
    }
    
}

