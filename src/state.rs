use crate::{board::BitBoard, castle::CastleRights, game::{BoardDelta, DeltaFlags}, pieces::{Piece, Pieces}, square::Square, team::Team, trace::MoveTrace};

#[derive(Copy, Clone)]
pub struct BoardState {
    pub en_passant: Option<Square>,
    pub next_hole: Option<Square>,
    pub hole_in_1: bool,
    pub wormholes: BitBoard,
    pub fullmoves: u16,
    pub halfmoves: u8,
    pub pieces: Pieces,
    pub castle: CastleRights,
    pub turn: Team,
}

impl BoardState {
    pub fn valid_moves(&self, sq: Square) -> BitBoard {
        crate::compute::compute(self, sq)
    }

    pub fn trace(&self, src: Square, dst: Square) -> Option<MoveTrace> {
        crate::trace::trace(self, src, dst)
    }

    /// Execute state changes.
    pub fn next(&self, delta: BoardDelta) -> Self {
        let mut next = *self;
        let flags = delta.get_flags();
        let src = delta.get_src_sq();
        let dst = delta.get_dst_sq();

        let moved_piece = next.pieces.piece_at_or_on_hole(src, self.wormholes);

        if let Some(side) = delta.castle_side() {
            // move the pieces accordingly.
            next.pieces.remove(self.castle.king_start(self.turn), BitBoard::EMPTY);
            next.pieces.remove(self.castle.rook_start(side, self.turn), BitBoard::EMPTY);
            next.pieces.insert_unchecked(self.castle.king_target(side, self.turn), Piece::King, self.turn);
            next.pieces.insert_unchecked(self.castle.rook_target(side, self.turn), Piece::Rook, self.turn);
            next.en_passant = None;
            next.halfmoves = 0;
        } else {
            // remove the src square
            next.pieces.remove(src, self.wormholes);

            // Remove the captured piece
            if flags.contains(DeltaFlags::IS_CAPTURE) {
                if let Some(ep_sq) = self.en_passant && flags.contains(DeltaFlags::IS_EN_PASSANT) {
                    next.pieces.remove(ep_sq, self.wormholes);
                } else {
                    next.pieces.remove(dst, self.wormholes);                
                }
            }

            // Update the en passant square.
            if flags.contains(DeltaFlags::IS_DOUBLE_PUSH) {
                if next.wormholes.has(dst) {
                    next.en_passant = Some(src + (self.turn.pawn_dir(), 0));
                } else {
                    next.en_passant = Some(dst + (-self.turn.pawn_dir(), 0))
                }
            }

            // Insert the moved piece at the destination square, respecting promotion.
            if let Some(promotion) = delta.get_promote_pc() {
                next.pieces.insert_unchecked(dst, promotion, self.turn);
            } else {
                if let Some(pc) = moved_piece {
                    next.pieces.insert_unchecked(dst, pc, self.turn);
                }
            }

            // reset halfmoves if necessary.
            if flags.contains(DeltaFlags::RESET_HALFMOVES) {
                next.halfmoves = 0;
            } else {
                next.halfmoves += 1;
            }
        }

        // advance fullmove counter if black is up to play.
        if self.turn == Team::Black {
            next.fullmoves += 1;
        }

        // update castle rights accordingly.
        next.castle.rights = delta.get_castle_rights();

        // update the wormholes
        if let Some(hole) = delta.get_hole_square() {
            if flags.contains(DeltaFlags::HOLE_PUSHED) {
                next.next_hole = Some(hole);
            } else if flags.contains(DeltaFlags::HOLE_POPPED) {
                next.next_hole = None;
                next.wormholes.set(hole);
            } else if flags.contains(DeltaFlags::HOLE_WILL_POP) {
                next.hole_in_1 = true;
            }
        }

        // advance turn
        next.turn = !self.turn;

        next
    }

    /// Undo the changes to get the previous position.
    pub fn prev(&self, delta: BoardDelta) -> Self {
        let mut prev = *self;
        let turn = !prev.turn;

        let flags = delta.get_flags();
        let src = delta.get_src_sq();
        let dst = delta.get_dst_sq();

        let moved_piece = prev.pieces.piece_at_or_on_hole(dst, prev.wormholes);

        // update the wormholes
        if let Some(hole) = delta.get_hole_square() {
            if flags.contains(DeltaFlags::HOLE_PUSHED) {
                prev.next_hole = None;
            } else if flags.contains(DeltaFlags::HOLE_POPPED) {
                prev.next_hole = Some(hole);
                prev.hole_in_1 = true;
                prev.wormholes.clear(hole);
            }
        }

        if let Some(side) = delta.castle_side() {
            // undo castling
            prev.pieces.remove(prev.castle.king_target(side, turn), BitBoard::new());
            prev.pieces.remove(prev.castle.rook_target(side, turn), BitBoard::new());
            prev.pieces.insert_unchecked(prev.castle.king_start(turn), Piece::King, turn);
            prev.pieces.insert_unchecked(prev.castle.rook_start(side, turn), Piece::Rook, turn);
        } else {
            prev.pieces.remove(dst, self.wormholes);
            
            // undo piece capture.
            if let Some(capture_pc) = delta.get_capture_pc() {
                if flags.contains(DeltaFlags::IS_EN_PASSANT) {
                    if let Some(ep_sq) = delta.get_prev_ep_sq() {
                        prev.pieces.insert_unchecked(ep_sq, capture_pc, self.turn);
                    }
                } else {
                    prev.pieces.insert_unchecked(dst, capture_pc, self.turn);
                }
            }

            // undo piece movement, respecting promotion.
            if delta.get_promote_pc().is_some() {
                prev.pieces.insert_unchecked(src, Piece::Pawn, turn);
            } else {
                if let Some(pc) = moved_piece {
                    prev.pieces.insert_unchecked(src, pc, turn);
                }
            }
        }

        prev.en_passant = delta.get_prev_ep_sq();
        prev.halfmoves = delta.get_prev_halfmoves();
        prev.turn = turn;

        if turn == Team::Black {
            prev.fullmoves -= 1;
        }

        prev
    }
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            en_passant: None,
            next_hole: None,
            hole_in_1: false,
            wormholes: BitBoard(0),
            fullmoves: 1,
            halfmoves: 0,
            pieces: Pieces::default(),
            castle: CastleRights::default(),
            turn: Team::White,
        }
    }
}

