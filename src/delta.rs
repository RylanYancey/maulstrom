use std::fmt;
use crate::{castle::Castle, pieces::Piece, square::Square};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct BoardDelta {
    /// The number of milliseconds the move took to be played.
    time: u32,

    /// Squares that had changes on the board.
    /// # Layout
    ///  - bits 0..=6: Source Square
    ///  - bits 7..=12: Destination Square
    ///  - bits 13..=18: Wormhole Square
    ///  - bits 19..=24: Square of pawn captured en-passant
    ///  - bits 25..=30: Previous En Passant Square
    ///  - bit 31: Whether the prev ep sq is Some.
    squares: u32,

    /// Relevant pieces and flags.
    /// # Layout
    ///  - bits 0,1,2: Capture Piece (6=None)
    ///  - bits 3,4,5: Promote Piece (6=None)
    ///  - bits 6,7,8: Piece Crushed by Wormhole Spawning (6=None)
    ///  - bits 9..=15: Previous Halfmove Count
    ///  - bit 16: IS_DOUBLE_PUSH (allows ep)
    ///  - bit 17: HALFMOVES_RESET
    ///  - bit 18: WORMHOLE_POPPED (wormhole on queue spawned)
    ///  - bit 19: WORMHOLE_PUSHED (wormhole pushed to queue)
    ///  - bit 20: WHITE_SHORT_CASTLE_FLIP
    ///  - bit 21: WHITE_LOST_CASTLE_FLIP
    ///  - bit 22: BLACK_SHORT_CASTLE_FLIP
    ///  - bit 23: BLACK_LONG_CASTLE_FLIP
    ///  - bit 24: IS_CAPTURE_EP
    ///  - bit 25: IS_CASTLE_LONG
    ///  - bit 26: IS_CASTLE_SHORT
    ///  - bit 27: WORMHOLE_IN_1 (wormhole will be popped next turn)
    ///  - bit 28: WAS_CHECK (whether the king was in check in the position the move was played in)
    ///  - bit 29: IS_CHECK (whether the king is in check in the resulting position)
    data: u32,
}

impl Default for BoardDelta {
    fn default() -> Self {
        Self {
            time: 0,
            squares: 0,
            data: 0x1FF
        }
    }
}

impl BoardDelta {
    pub fn get_capture_pc(&self) -> Option<Piece> {
        Piece::from_u8((self.data & 0b111) as u8)
    }

    pub fn set_capture_pc(&mut self, pc: Piece) {
        self.data &= !0b111;
        self.data |= pc.to_u8() as u32;
    }
    
    pub fn get_promote_pc(&self) -> Option<Piece> {
        Piece::from_u8(((self.data >> 3) & 0b111) as u8)
    }

    pub fn set_promote_pc(&mut self, pc: Piece) {
        let bits = match pc {
            Piece::Bishop => 0,
            Piece::Knight => 1,
            Piece::Queen => 2,
            Piece::Rook => 3,
            _ => 6,
        };

        self.data &= !0x38;
        self.data |= bits << 3;
    }

    pub fn get_ep_capture_sq(&self) -> Option<Square> {
        if self.data & (1 << 24) != 0 {
            Some(Square::from_index(((self.squares >> 19) & 0x3F) as usize))
        } else {
            None
        }
    }

    pub fn set_ep_capture_sq(&mut self, sq: Square) {
        self.data |= 1 << 24;
        self.squares &= !(0x3F << 19);
        self.squares |= (sq.to_index() as u32) << 19;
    }

    pub fn is_double_push(&self) -> bool {
        self.data & (1 << 16) != 0
    }

    pub fn set_is_double_push(&mut self) {
        self.data |= 1 << 16;
    }

    pub fn get_castle_side(&self) -> Option<Castle> {
        if self.data & 1 << 26 != 0 { return Some(Castle::Short) };
        if self.data & 1 << 25 != 0 { return Some(Castle::Long) };
        None
    }

    pub fn set_is_castle(&mut self, side: Castle) {
        match side {
            Castle::Short => self.data |= 1 << 26,
            Castle::Long => self.data |= 1 << 25,
        }
    }

    pub fn get_castle_deltas(&self) -> u8 {
        ((self.data >> 20) & 0b1111) as u8
    }

    pub fn set_castle_deltas(&mut self, prev: u8, next: u8) {
        self.data &= !0b1111 << 20;
        self.data |= ((prev ^ next) as u32) << 20;
    }

    pub fn get_src_sq(&self) -> Square {
        Square::from_index((self.squares & 0b111111) as usize)
    }

    pub fn set_src_sq(&mut self, sq: Square) {
        self.squares &= !0b111111;
        self.squares |= sq.to_index() as u32;
    }

    pub fn get_dst_sq(&self) -> Square {
        Square::from_index(((self.squares >> 6) & 0b111111) as usize)
    }

    pub fn set_dst_sq(&mut self, sq: Square) {
        self.squares &= !(0b111111 << 6);
        self.squares |= (sq.to_index() as u32) << 6;
    }

    pub fn get_prev_ep_sq(&self) -> Option<Square> {
        if self.squares & 1 << 31 != 0 {
            Some(Square::from_index(((self.squares >> 25) & 0b111111) as usize))
        } else {
            None
        }
    }

    pub fn set_prev_ep_sq(&mut self, sq: Square) {
        self.squares &= !(0b111111 << 25);
        self.squares |= (sq.to_index() as u32) << 25;
        self.squares |= 1 << 31;
    }

    pub fn get_wormhole_sq(&self) -> Square {
        Square::from_index(((self.squares >> 18) & 0b111111) as usize)
    }

    pub fn set_wormhole_sq(&mut self, sq: Square) {
        self.squares &= !(0b111111 << 18);
        self.squares |= (sq.to_index() as u32) << 18;
    }

    pub fn get_prev_halfmoves(&self) -> u8 {
        ((self.data >> 9) & 0x3F) as u8
    }

    pub fn set_prev_halfmoves(&mut self, halfmoves: u8) {
        self.data &= !(0x3F << 9);
        self.data |= (halfmoves as u32) << 9;
    }

    pub fn is_popped_wormhole(&self) -> bool {
        self.data & (1 << 18) != 0
    }

    pub fn set_popped_wormhole(&mut self) {
        self.data |= 1 << 18
    }

    pub fn is_pushed_wormhole(&self) -> bool {
        self.data & (1 << 19) != 0
    }

    pub fn set_pushed_wormhole(&mut self) {
        self.data |= 1 << 19;
    }

    pub fn get_crushed_pc(&self) -> Option<Piece> {
        Piece::from_u8(((self.data >> 6) & 0b111) as u8)
    }

    pub fn set_crushed_pc(&mut self, pc: Piece) {
        self.data &= !(0b1111 << 6);
        self.data |= (pc.to_u8() as u32) << 6;
    }

    pub fn is_resets_halfmoves(&self) -> bool {
        self.data & (1 << 17) != 0
    }

    pub fn set_resets_halfmoves(&mut self) {
        self.data |= 1 << 17;
    }

    pub fn is_wormhole_in_1(&self) -> bool {
        self.data & 1 << 27 != 0
    }

    pub fn set_wormhole_in_1(&mut self) {
        self.data |= 1 << 27;
    }

    pub fn was_check(&self) -> bool {
        self.data & (1 << 28) != 0
    }

    pub fn set_was_check(&mut self) {
        self.data |= 1 << 28
    }

    pub fn is_check(&self) -> bool {
        self.data & (1 << 29) != 0
    }

    pub fn set_is_check(&mut self) {
        self.data |= 1 << 29
    }
}

impl fmt::Debug for BoardDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoardDelta")
            .field("src_sq", &self.get_src_sq())
            .field("dst_sq", &self.get_dst_sq())
            .field("capture_pc", &self.get_capture_pc())
            .field("promote_pc", &self.get_promote_pc())
            .field("resets_halfmoves", &self.is_resets_halfmoves())
            .field("ep_capture_sq", &self.get_ep_capture_sq())
            .field("prev_ep_sq", &self.get_prev_ep_sq())
            .field("prev_halfmoves", &self.get_prev_halfmoves())
            .field("is_double_push", &self.is_double_push())
            .field("castle_side", &self.get_castle_side())
            .field("castle_deltas", &self.get_castle_deltas())
            .field("crushed_pc", &self.get_crushed_pc())
            .field("is_wormhole_in_1", &self.is_wormhole_in_1())
            .field("is_pushed_wormhole", &self.is_pushed_wormhole())
            .field("is_popped_wormhole", &self.is_popped_wormhole())
            .field("wormhole_sq", &self.get_wormhole_sq())
            .finish()
    }
}