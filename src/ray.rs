use crate::{board::BitBoard, square::Square, cached::*};


/// For a ray from src to dst, where src is below dst, shorten
/// the ray if a square in the occupied mask blocks it.
#[inline]
pub const fn truncate_lt(ray: u64, occ: u64) -> u64 {
    let o = ray & occ;
    if o == 0 { ray } else { 
        ray & (!0u64 >> (63 - o.trailing_zeros())) 
    }
}

/// For a ray from src to dst, where src is above dst, shorten
/// the ray if a square in the occupied mask blocks it.
#[inline]
pub const fn truncate_gt(ray: u64, occ: u64) -> u64 {
    let o = ray & occ;
    if o == 0 { ray } else { 
        ray & (!0u64 << (63 - o.leading_zeros())) 
    }
}

#[inline]
pub const fn pos_zero(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_lt(RAY_POS_ZERO_EXCLUSIVE[sq.to_index()], occ.0))
}

#[inline]
pub const fn neg_zero(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_gt(RAY_NEG_ZERO_EXCLUSIVE[sq.to_index()], occ.0))
}

#[inline]
pub const fn zero_pos(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_lt(RAY_ZERO_POS_EXCLUSIVE[sq.to_index()], occ.0))
}

#[inline]
pub const fn zero_neg(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_gt(RAY_ZERO_NEG_EXCLUSIVE[sq.to_index()], occ.0))
}

#[inline]
pub const fn pos_pos(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_lt(RAY_POS_POS_EXCLUSIVE[sq.to_index()], occ.0))
}

#[inline]
pub const fn neg_neg(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_gt(RAY_NEG_NEG_EXCLUSIVE[sq.to_index()], occ.0))
}

#[inline]
pub const fn pos_neg(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_lt(RAY_POS_NEG_EXCLUSIVE[sq.to_index()], occ.0))
}

#[inline]
pub const fn neg_pos(sq: Square, occ: BitBoard) -> BitBoard {
    BitBoard(truncate_gt(RAY_NEG_POS_EXCLUSIVE[sq.to_index()], occ.0))
}

pub const fn truncate_lt_if_hit(ray: u64, occ: u64) -> Option<u64> {
    let o = ray & occ;
    if o == 0 { None } else {
        Some(ray & (!0u64 >> 63 - o.trailing_zeros()))
    }
}

pub const fn truncate_gt_if_hit(ray: u64, occ: u64) -> Option<u64> {
    let o = ray & occ;
    if o == 0 { None } else {
        Some(ray & (!0u64 << 63 - o.leading_zeros()))
    }
}

#[inline]
pub const fn pos_zero_if_hit(sq: Square, occ: BitBoard) -> Option<BitBoard> {
    if let Some(ray) = truncate_lt_if_hit(RAY_POS_ZERO_EXCLUSIVE[sq.to_index()], occ.0) {
        Some(BitBoard(ray))
    } else {
        None
    }
}

#[inline]
pub const fn neg_zero_if_hit(sq: Square, occ: BitBoard) -> Option<BitBoard> {
    if let Some(ray) = truncate_gt_if_hit(RAY_NEG_ZERO_EXCLUSIVE[sq.to_index()], occ.0) {
        Some(BitBoard(ray))
    } else {
        None
    }
}

#[inline]
pub const fn zero_pos_if_hit(sq: Square, occ: BitBoard) -> Option<BitBoard> {
    if let Some(ray) = truncate_lt_if_hit(RAY_ZERO_POS_EXCLUSIVE[sq.to_index()], occ.0) {
        Some(BitBoard(ray))
    } else {
        None
    }
}

#[inline]
pub const fn zero_neg_if_hit(sq: Square, occ: BitBoard) -> Option<BitBoard> {
    if let Some(ray) = truncate_gt_if_hit(RAY_ZERO_NEG_EXCLUSIVE[sq.to_index()], occ.0) {
        Some(BitBoard(ray))
    } else {
        None
    }
}

#[inline]
pub const fn pos_pos_if_hit(sq: Square, occ: BitBoard) -> Option<BitBoard> {
    if let Some(ray) = truncate_lt_if_hit(RAY_POS_POS_EXCLUSIVE[sq.to_index()], occ.0) {
        Some(BitBoard(ray))
    } else {
        None
    }
}

#[inline]
pub const fn neg_neg_if_hit(sq: Square, occ: BitBoard) -> Option<BitBoard> {
    if let Some(ray) = truncate_gt_if_hit(RAY_NEG_NEG_EXCLUSIVE[sq.to_index()], occ.0) {
        Some(BitBoard(ray))
    } else {
        None
    }
}

#[inline]
pub const fn pos_neg_if_hit(sq: Square, occ: BitBoard) -> Option<BitBoard> {
    if let Some(ray) = truncate_lt_if_hit(RAY_POS_NEG_EXCLUSIVE[sq.to_index()], occ.0) {
        Some(BitBoard(ray))
    } else {
        None
    }
}

#[inline]
pub const fn neg_pos_if_hit(sq: Square, occ: BitBoard) -> Option<BitBoard> {
    if let Some(ray) = truncate_gt_if_hit(RAY_NEG_POS_EXCLUSIVE[sq.to_index()], occ.0) {
        Some(BitBoard(ray))
    } else {
        None
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Ray {
    PosPos,
    NegNeg,
    PosNeg,
    NegPos,
    PosZero,
    NegZero,
    ZeroPos,
    ZeroNeg,
}

impl Ray {
    pub const fn cast(&self, src: Square, occ: BitBoard) -> BitBoard {
        match self {
            Self::PosPos => pos_pos(src, occ),
            Self::NegNeg => neg_neg(src, occ),
            Self::PosNeg => pos_neg(src, occ),
            Self::NegPos => neg_pos(src, occ),
            Self::PosZero => pos_zero(src, occ),
            Self::NegZero => neg_zero(src, occ),
            Self::ZeroPos => zero_pos(src, occ),
            Self::ZeroNeg => zero_neg(src, occ)
        }
    }

    pub const fn cast_if_hit(&self, src: Square, occ: BitBoard) -> Option<BitBoard> {
        match self {
            Self::PosPos => pos_pos_if_hit(src, occ),
            Self::NegNeg => neg_neg_if_hit(src, occ),
            Self::PosNeg => pos_neg_if_hit(src, occ),
            Self::NegPos => neg_pos_if_hit(src, occ),
            Self::PosZero => pos_zero_if_hit(src, occ),
            Self::NegZero => neg_zero_if_hit(src, occ),
            Self::ZeroPos => zero_pos_if_hit(src, occ),
            Self::ZeroNeg => zero_neg_if_hit(src, occ)
        }
    }
}