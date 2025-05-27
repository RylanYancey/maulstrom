use crate::{board::BitBoard, castle::CastleRights, pieces::{Piece, Pieces}, square::Square, team::Team, trace::MoveTrace, delta::BoardDelta};

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

    /// Execute changes.
    pub fn next(&self, delta: BoardDelta) -> Self {
        let mut next = *self;
        let src = delta.get_src_sq();
        let dst = delta.get_dst_sq();

        let moved_piece = self.pieces.piece_at_or_on_hole(src, self.wormholes);

        if let Some(side) = delta.get_castle_side() {
            next.pieces.remove(self.castle.king_start(self.turn), self.wormholes);
            next.pieces.remove(self.castle.rook_start(side, self.turn), self.wormholes);
            next.pieces.insert(self.castle.king_target(side, self.turn), Piece::King, self.turn, self.wormholes);
            next.pieces.insert(self.castle.rook_target(side, self.turn), Piece::Rook, self.turn, self.wormholes);
        } else {
            next.pieces.remove(src, self.wormholes);

            // remove captured piece
            if let Some(_) = delta.get_capture_pc() {
                if let Some(capture_sq) = delta.get_ep_capture_sq() {
                    next.pieces.remove(capture_sq, self.wormholes);
                } else {
                    next.pieces.remove(dst, self.wormholes);
                }
            }

            // handle promotion and piece movement
            if let Some(promote_pc) = delta.get_promote_pc() {
                next.pieces.insert(dst, promote_pc, self.turn, self.wormholes);
            } else {
                if let Some(moved_pc) = moved_piece {
                    next.pieces.insert(dst, moved_pc, next.turn, self.wormholes);
                }
            }

            // update ep square.
            if delta.is_double_push() {
                if self.wormholes.has(dst) {
                    next.en_passant = Some(src + (self.turn.pawn_dir(), 0))
                } else {
                    next.en_passant = Some(dst + (-self.turn.pawn_dir(), 0))
                }
            }
        }

        if delta.is_wormhole_in_1() {
            next.hole_in_1 = true;
        }

        if delta.is_pushed_wormhole() {
            next.next_hole = Some(delta.get_wormhole_sq());
        }

        if delta.is_popped_wormhole() {
            let hole_sq = delta.get_wormhole_sq();
            debug_assert_eq!(self.next_hole, Some(hole_sq), "[E998 (invalid hole state)]");
            next.wormholes.set(hole_sq);
            next.next_hole = None;
        }

        next.castle.rights ^= delta.get_castle_deltas();

        // update halfmove counter
        if delta.is_resets_halfmoves() {
            next.halfmoves = 0;
        } else {
            next.halfmoves += 1;
        }

        // Fullmoves increments when black moves.
        if self.turn == Team::Black {
            next.fullmoves += 1;
        }

        next
    }

    /// Undo the changes to get the previous position.
    pub fn prev(&self, delta: BoardDelta) -> Self {
        let mut prev = *self;
        prev.turn = !self.turn;

        let src = delta.get_src_sq();
        let dst = delta.get_dst_sq();

        let moved_piece = prev.pieces.piece_at_or_on_hole(dst, prev.wormholes);

        if let Some(side) = delta.get_castle_side() {
            prev.pieces.remove(self.castle.king_target(side, prev.turn), prev.wormholes);
            prev.pieces.remove(self.castle.rook_target(side, prev.turn), prev.wormholes);
            prev.pieces.insert(self.castle.king_start(prev.turn), Piece::King, prev.turn, prev.wormholes);
            prev.pieces.insert(self.castle.rook_start(side, prev.turn), Piece::Rook, prev.turn, prev.wormholes);
        } else {
            prev.pieces.remove(dst, prev.wormholes);

            if let Some(capture_pc) = delta.get_capture_pc() {
                if let Some(ep_capture_sq) = delta.get_ep_capture_sq() {
                    prev.pieces.insert(ep_capture_sq, Piece::Pawn, self.turn, prev.wormholes);
                } else {
                    prev.pieces.insert(dst, capture_pc, self.turn, prev.wormholes);
                }
            }

            if let Some(promote_pc) = delta.get_promote_pc() {
                prev.pieces.insert(src, promote_pc, prev.turn, prev.wormholes);
            } else {
                if let Some(moved_pc) = moved_piece {
                    prev.pieces.insert(src, moved_pc, prev.turn, prev.wormholes);
                }
            }
        }

        prev.en_passant = delta.get_prev_ep_sq();

        if delta.is_pushed_wormhole() {
            prev.next_hole = None;
        } else if delta.is_popped_wormhole() {
            let hole_sq = delta.get_wormhole_sq();
            prev.wormholes.clear(hole_sq);
            prev.next_hole = Some(hole_sq);
            prev.hole_in_1 = true;
        }

        prev.castle.rights ^= delta.get_castle_deltas();
        prev.halfmoves = delta.get_prev_halfmoves();

        if self.turn == Team::White {
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

