
use crate::{board::BitBoard, cached::*, delta::*, castle::{can_castle, Castle}, game::BoardDelta, pieces::Piece, square::Square, state::BoardState, team::Team};

#[derive(Copy, Clone)]
pub struct MoveTrace {
    /// If the move can be done through wormholes, this will be Some((in_sq, out_sq))
    pub route: Option<(Square, Square)>,

    /// Whether the move is a capture of a piece.
    pub captures: Option<Piece>,

    /// Whether the move is a capture en-passant.
    pub is_capture_en_passant: Option<Square>,

    /// Whether the move allows en passant
    pub allows_en_passant: Option<Square>,

    /// Whether the move requires a pawn promotion.
    pub requires_promotion: bool,

    /// Whether the move is a king move (loses both castling sides)
    pub is_king_move: bool,

    /// If it is castle, the side that it is castling on.
    pub is_castle: Option<Castle>,

    /// Whether (long, short) castle is lost.
    /// We only use this when a rook moves or is captured, all king
    /// moves lose castling in both directions.
    pub loses_castle: Option<Castle>,

    /// Whether the move takes castle from the opponent.
    /// Occurs when you capture an opponents' rook that had castle rights.
    pub takes_castle: Option<Castle>,
}

impl Default for MoveTrace {
    fn default() -> Self {
        Self {
            route: None,
            captures: None,
            is_capture_en_passant: None,
            allows_en_passant: None,
            requires_promotion: false,
            is_king_move: false,
            is_castle: None,
            loses_castle: None,
            takes_castle: None,
        }
    }
}

pub fn trace(state: &BoardState, src: Square, dst: Square) -> Option<MoveTrace> {
    let team = state.turn;
    let holes = state.wormholes;
    let mut occupied = state.pieces.occupied();
    if holes.intersects(occupied) {
        occupied |= holes;
    }

    let mut on_team = state.pieces.on_team(team);
    if holes.intersects(on_team) {
        on_team |= holes;
    }
    
    // cannot move out-of-turn or an empty square.
    if !on_team.has(src) {
        return None;
    }

    let mut sqs = BitBoard::new() | src;
    if holes.has(src) {
        // cannot move from a wormhole to a wormhole.
        if holes.has(dst) {
            return None;
        }

        sqs |= holes;
    }

    let dsts = if holes.has(dst) { holes } else { BitBoard::new() | dst };

    let takes_castle = if state.pieces.get(Piece::Rook, !state.turn).intersects(dsts) {
        if dst == state.castle.rook_start(Castle::Long, !state.turn) {
            Some(Castle::Long)
        } else if dst == state.castle.rook_start(Castle::Short, !state.turn) {
            Some(Castle::Short)
        } else {
            None
        }
    } else {
        None
    };

    let captures = state.pieces.piece_at_or_on_hole(dst, holes);

    if state.pieces.get(Piece::King, team).intersects(sqs) {
        let defense = crate::defense::defense(state);

        for side in [Castle::Short, Castle::Long] {
            if can_castle(side, team, state.castle, defense, occupied, src) && (
                dst == state.castle.rook_start(side, team) ||
                dst == state.castle.king_target(side, team)
            ) {
                return Some(
                    MoveTrace {
                        is_castle: Some(side),
                        is_king_move: true,
                        ..MoveTrace::default()
                    }
                )
            }
        }

        // cannot move to start square. (chess960 castling this is possible)
        if src == dst {
            return None;
        }

        // cannot move to piece on own team. (unless castling from rook->king)
        if on_team.intersects(dsts) {
            return None;
        }

        let moves = BitBoard(KING_MOVES[src.to_index()]) & !(on_team | defense);

        if moves.intersects(dsts) {
            return Some(
                MoveTrace {
                    is_king_move: true,
                    takes_castle,
                    captures,
                    ..MoveTrace::default()
                }
            )
        } else if moves.intersects(holes) && holes.intersects(dsts) {
            return Some(
                MoveTrace {
                    is_king_move: true,
                    route: Some(((moves & holes).first().unwrap(), dst)),
                    takes_castle,
                    captures,
                    ..MoveTrace::default()
                }
            )
        } else {
            if holes.has(src) {
                for out_sq in holes.without(src) {
                    if (BitBoard(KING_MOVES[out_sq.to_index()]) & !(on_team | defense)).has(dst) {
                        return Some(
                            MoveTrace {
                                is_king_move: true,
                                route: Some((src, out_sq)),
                                takes_castle,
                                captures,
                                ..MoveTrace::default()
                            }
                        )
                    }
                }
            }

            return None
        }
    }

    let blockable = crate::blockable::blockable(state, src);
    if !blockable.has(dst) {
        return None;
    }

    if src == dst {
        return None;
    }

    if on_team.intersects(dsts) {
        if state.pieces.get(Piece::Rook, team).intersects(sqs) {
            if let Some(king) = state.pieces.get(Piece::King, team).first() {
                if dst == king {
                    for side in [Castle::Long, Castle::Short] {
                        if state.castle.has(side, team) && src == state.castle.rook_start(side, team) {
                            let defense = crate::defense::defense(state);
                            if can_castle(side, team, state.castle, defense, occupied, king) {
                                return Some(
                                    MoveTrace {
                                        route: None,
                                        is_castle: Some(side),
                                        is_king_move: true,
                                        ..Default::default()
                                    }
                                )
                            }
                        }
                    }
                }
            }
        }

        return None;
    }

    if state.pieces.get(Piece::Pawn, team).intersects(sqs) {
        let requires_promotion = dsts.intersects(BitBoard::new().with_rank((!team).back_rank()));
        let dir = team.pawn_dir();
        let pawn_rank = team.pawn_rank();

        if holes.has(src) {
            for out_sq in holes {
                let takes = match team {
                    Team::White => BitBoard(WHITE_PAWN_ATTACKS[out_sq.to_index()]),
                    Team::Black => BitBoard(BLACK_PAWN_ATTACKS[out_sq.to_index()]),
                };

                if takes.intersects(dsts) {
                    if dsts.intersects(state.pieces.on_team(!team)) {
                        return Some(
                            MoveTrace {
                                route: (out_sq != src).then(|| (src, out_sq)),
                                captures,
                                requires_promotion,
                                ..MoveTrace::default()
                            }
                        )
                    } else {
                        if let Some(ep_sq) = state.en_passant {
                            if dsts.has(ep_sq) {
                                let ep_pawn = ep_sq + ((!state.turn).pawn_dir(), 0);

                                return Some(
                                    MoveTrace {
                                        route: (out_sq != src).then(|| (src, out_sq)),
                                        is_capture_en_passant: Some(ep_pawn),
                                        captures,
                                        takes_castle,
                                        requires_promotion,
                                        ..MoveTrace::default()
                                    }
                                )
                            }
                        }
                    }
                }
            }

            if dsts.intersects(occupied) {
                return None;
            }

            if holes.intersects(BitBoard::new().with_rank(pawn_rank)) {
                for out_sq in holes {
                    let one = out_sq + (dir, 0);
                    if !occupied.has(one) {
                        if one.is_in_bounds() && dsts.has(one) {
                            return Some(
                                MoveTrace {
                                    route: (out_sq != src).then(|| (src, out_sq)),
                                    requires_promotion,
                                    ..MoveTrace::default()
                                }
                            )
                        }

                        let two = one + (dir, 0);
                        if two.is_in_bounds() && dsts.has(two) {
                            return Some(
                                MoveTrace {
                                    route: (out_sq != src).then(|| (src, out_sq)),
                                    requires_promotion,
                                    allows_en_passant: Some(one),
                                    ..MoveTrace::default()
                                }
                            )
                        }
                    }
                }
            } else {
                for out_sq in holes {
                    let one = src + (dir, 0);
                    if one.is_in_bounds() && dsts.has(one) {
                        return Some(
                            MoveTrace {
                                route: (out_sq != src).then(|| (src, out_sq)),
                                requires_promotion,
                                ..MoveTrace::default()
                            }
                        )
                    }
                }
            }
        } else {
            let one = src + (dir, 0);

            if one.is_in_bounds() && !occupied.has(one) {
                if dsts.has(one) {
                    return Some(
                        MoveTrace {
                            requires_promotion,
                            ..MoveTrace::default()
                        }
                    )
                }

                if src.rank == pawn_rank {
                    let two = one + (dir, 0);

                    if two.is_in_bounds() && dsts.has(two) {
                        return Some(
                            MoveTrace {
                                allows_en_passant: Some(one),
                                requires_promotion,
                                ..MoveTrace::default()
                            }
                        )
                    }

                    if holes.has(one) {
                        for out_sq in holes.without(one) {
                            let out_one = out_sq + (dir, 0);

                            if dsts.has(out_one) {
                                return Some(
                                    MoveTrace {
                                        route: Some((one, out_sq)),
                                        allows_en_passant: Some(one),
                                        requires_promotion,
                                        ..MoveTrace::default()
                                    }
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    if state.pieces.get(Piece::Knight, team).intersects(sqs) {
        let moves = BitBoard(KNIGHT_MOVES[src.to_index()]) & !on_team;

        if moves.intersects(dsts) {
            return Some(
                MoveTrace {
                    takes_castle,
                    captures,
                    ..MoveTrace::default()
                }
            )
        } else if moves.intersects(holes) && holes.intersects(dsts) {
            return Some(
                MoveTrace {
                    route: Some(((moves & holes).first().unwrap(), dst)),
                    takes_castle,
                    captures,
                    ..MoveTrace::default()
                }
            )
        } else {
            if holes.has(src) {
                for out_sq in holes.without(src) {
                    if (BitBoard(KNIGHT_MOVES[out_sq.to_index()]) & !on_team).has(dst) {
                        return Some(
                            MoveTrace {
                                route: Some((src, out_sq)),
                                takes_castle,
                                captures,
                                ..MoveTrace::default()
                            }
                        )
                    }
                }
            }

            return None
        }
    }

    if state.pieces.get(Piece::Queen, team).intersects(sqs) {
        if let Some(trace) = ray(occupied, pos_zero, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, neg_zero, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, zero_neg, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, zero_pos, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, pos_pos, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, pos_neg, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, neg_neg, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, neg_pos, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        return None;
    }

    if state.pieces.get(Piece::Bishop, team).intersects(sqs) {
        if let Some(trace) = ray(occupied, pos_zero, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, neg_zero, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, zero_neg, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, zero_pos, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        return None;
    }

    if state.pieces.get(Piece::Rook, team).intersects(sqs) {
        if let Some(trace) = ray(occupied, pos_pos, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, pos_neg, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, neg_neg, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        if let Some(trace) = ray(occupied, neg_pos, captures, takes_castle, holes, dsts, src) { return Some(trace) }
        return None;
    }

    None
}

fn ray(
    occupied: BitBoard,
    delta: fn(Square, BitBoard) -> BitBoard,
    captures: Option<Piece>,
    takes_castle: Option<Castle>,
    holes: BitBoard,
    dsts: BitBoard,
    src: Square,
) -> Option<MoveTrace> {
    if holes.has(src) {
        for out_sq in holes {
            let ray = delta(out_sq, occupied);
            if ray.intersects(dsts) {
                return Some(
                    MoveTrace {
                        route: (src != out_sq).then(|| (src, out_sq)),
                        captures,
                        takes_castle,
                        ..MoveTrace::default()
                    }
                )
            }
        }
    } else {
        let ray = delta(src, occupied);

        if ray.intersects(dsts) {
            return Some(
                MoveTrace {
                    captures,
                    takes_castle,
                    ..MoveTrace::default()
                }
            )
        } else {
            if let Some(in_sq) = (ray & holes).first() && !occupied.has(in_sq) {
                for out_sq in holes & !ray {
                    let ray = delta(out_sq, occupied);  

                    if ray.intersects(dsts) {
                        return Some(
                            MoveTrace {
                                route: Some((in_sq, out_sq)),
                                captures,
                                takes_castle,
                                ..MoveTrace::default()
                            }
                        )
                    }
                }
            }
        }
    }

    None
}