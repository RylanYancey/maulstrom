use crate::{board::BitBoard, square::Square, cached::*};


/// For a ray from src to dst, where src is below dst, shorten
/// the ray if a square in the occupied mask blocks it.
pub const fn truncate_lt(ray: u64, occ: u64) -> u64 {
    let o = ray & occ;
    if o == 0 { ray } else { 
        ray & (!0u64 >> (63 - o.trailing_zeros())) 
    }
}

/// For a ray from src to dst, where src is above dst, shorten
/// the ray if a square in the occupied mask blocks it.
pub const fn truncate_gt(ray: u64, occ: u64) -> u64 {
    let o = ray & occ;
    if o == 0 { ray } else { 
        ray & (!0u64 << (63 - o.leading_zeros())) 
    }
}

pub fn pos_zero(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_lt(RAY_POS_ZERO_EXCLUSIVE[sq.to_index()], occ.0))
}

pub fn neg_zero(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_gt(RAY_NEG_ZERO_EXCLUSIVE[sq.to_index()], occ.0))
}

pub fn zero_pos(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_lt(RAY_ZERO_POS_EXCLUSIVE[sq.to_index()], occ.0))
}

pub fn zero_neg(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_gt(RAY_ZERO_NEG_EXCLUSIVE[sq.to_index()], occ.0))
}

pub fn pos_pos(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_lt(RAY_POS_POS_EXCLUSIVE[sq.to_index()], occ.0))
}

pub fn neg_neg(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_gt(RAY_NEG_NEG_EXCLUSIVE[sq.to_index()], occ.0))
}

pub fn pos_neg(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_lt(RAY_POS_NEG_EXCLUSIVE[sq.to_index()], occ.0))
}

pub fn neg_pos(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_gt(RAY_NEG_POS_EXCLUSIVE[sq.to_index()], occ.0))
}
