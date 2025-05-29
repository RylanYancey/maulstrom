
use std::ops::Not;

use crate::square::Rank;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Team {
    White,
    Black,
}

impl Team {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Black => "black",
        }
    }

    pub const fn pawn_dir(&self) -> i8 {
        match self {
            Self::White => 1,
            Self::Black => -1
        }
    }

    pub const fn back_rank(&self) -> Rank {
        match self {
            Self::White => Rank::First,
            Self::Black => Rank::Eighth,
        }
    }

    pub const fn back_rank_u8(&self) -> u8 {
        match self {
            Self::White => 0,
            Self::Black => 7,
        }
    }

    pub const fn pawn_rank(&self) -> Rank {
        match self {
            Self::White => Rank::Second,
            Self::Black => Rank::Seventh,
        }
    }

    pub const fn pawn_rank_u8(&self) -> u8 {
        match self {
            Self::White => 1,
            Self::Black => 6,
        }
    }
}

impl Not for Team {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}