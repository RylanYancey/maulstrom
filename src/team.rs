
use std::ops::Not;

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

    pub const fn back_rank(&self) -> i8 {
        match self {
            Self::White => 0,
            Self::Black => 7,
        }
    }

    pub const fn pawn_rank(&self) -> i8 {
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