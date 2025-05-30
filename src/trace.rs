
use crate::{board::BitBoard, cached::*, ray::*, castle::{can_castle, Castle}, pieces::Piece, square::Square, state::BoardState, team::Team};

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
 
pub fn trace(state: &BoardState, src: Square, dst: Square, defense: Option<BitBoard>) -> Option<MoveTrace> {
    // cannot move out-of-turn.
    if !state.pieces.on_team(state.turn).has(src) {
        return None;
    }

    let wormholes = state.wormholes;

    if let Some(pc) = state.pieces.piece_at_or_on_hole(src, wormholes) {
        let friendly = state.pieces.on_team(state.turn);
        let dsts = BitBoard::from(dst).transmit(wormholes);
        let srcs = BitBoard::from(src).transmit(wormholes);
        let turn = state.turn;
        let occupied = state.pieces.occupied().transmit(wormholes);
        let holes_are_occ = occupied.intersects(wormholes);
        let captures = state.pieces.piece_at_or_on_hole(dst, wormholes);
        let loses_castle = state.castle.move_loses_castle(srcs, turn);
        let takes_castle = state.castle.capture_takes_castle(dsts, !turn);

        match pc {
            Piece::King => {
                let defense = defense.unwrap_or_else(|| crate::defense::defense(state));

                if dst.rank() == turn.back_rank() && src.rank() == turn.back_rank() {
                    for side in [Castle::Long, Castle::Short] {
                        if src == state.castle.king_start(turn) {
                            if can_castle(side, turn, state.castle, defense, occupied, src) && (
                                dst == state.castle.rook_target(side, turn) || 
                                dst == state.castle.king_target(side, turn)
                            ) {
                                return Some(MoveTrace {
                                    is_king_move: true,
                                    is_castle: Some(side),
                                    ..MoveTrace::default()
                                })
                            }
                        }
                    }
                }

                if wormholes.has(src) {
                    for out_sq in wormholes {
                        let mv = out_sq.king_moves() & !(friendly | defense | wormholes);
                        if mv.has(dst) {
                            return Some(MoveTrace {
                                route: (out_sq != src).then(|| (src, out_sq)),
                                is_king_move: true,
                                ..MoveTrace::default()
                            })
                        }
                    }
                } else {
                    let mv = src.king_moves() & !(friendly | defense);
                    if mv.intersects(dsts) {
                        return Some(MoveTrace {
                            is_king_move: true,
                            ..MoveTrace::default()
                        })
                    }
                }
            },
            Piece::Knight => {
                let blockable = crate::blockable::blockable(src, state);
                if wormholes.has(src) {
                    for out_sq in wormholes {
                        if ((out_sq.knight_moves() & !friendly) & blockable).intersects(dsts) {
                            return Some(MoveTrace {
                                route: (out_sq != src).then(|| (src, out_sq)),
                                takes_castle,
                                captures,
                                ..MoveTrace::default()
                            })
                        }
                    }
                } else {
                    if ((src.knight_moves() & !friendly) & blockable).intersects(dsts) {
                        return Some(MoveTrace {
                            takes_castle,
                            captures,
                            ..MoveTrace::default()
                        })
                    }
                }
            },
            Piece::Bishop => {
                let blockable = crate::blockable::blockable(src, state);
                if wormholes.has(src) {
                    for out_sq in wormholes {
                        let diag = (out_sq.bishop_moves(occupied) & blockable) & !friendly;
                        if diag.has(dst) {
                            return Some(MoveTrace {
                                route: (src != out_sq).then(|| (src, out_sq)),
                                captures,
                                takes_castle,
                                ..Default::default()
                            })
                        }
                    }
                } else {
                    let moves = src.bishop_moves(occupied);
                    if ((moves & !friendly) & blockable).intersects(dsts) {
                        return Some(MoveTrace {
                            takes_castle,
                            captures,
                            ..MoveTrace::default()
                        })
                    }

                    if !holes_are_occ {
                        for in_sq in moves & wormholes {
                            if let Some(ray) = src.diag_ray(in_sq) {
                                for out_sq in wormholes {
                                    if ((ray.cast(out_sq, occupied) & !friendly) & blockable).intersects(dsts) {
                                        return Some(MoveTrace {
                                            route: Some((in_sq, out_sq)),
                                            takes_castle,
                                            captures,
                                            ..MoveTrace::default()
                                        })
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Piece::Rook => {
                let blockable = crate::blockable::blockable(src, state);

                let king_sq = state.castle.king_start(turn);
                if dsts.has(king_sq) && state.pieces.get(Piece::King, turn).has(king_sq) {
                    if blockable == BitBoard(!0) {
                        for side in [Castle::Long, Castle::Short] {
                            if src == state.castle.rook_start(side, turn) {
                                let defense = defense.unwrap_or_else(|| crate::defense::defense(state));
                                if can_castle(side, turn, state.castle, defense, occupied, king_sq) {
                                    return Some(MoveTrace {
                                        is_castle: Some(side),
                                        is_king_move: true,
                                        ..Default::default()
                                    })
                                }
                            }
                        }
                    }
                } else {
                    if wormholes.has(src) {
                        for out_sq in wormholes {
                            let ortho = (out_sq.rook_moves(occupied) & !friendly) & blockable;
                            if ortho.has(dst) {
                                return Some(MoveTrace {
                                    route: (src != out_sq).then(|| (src, out_sq)),
                                    captures,
                                    takes_castle,
                                    loses_castle,
                                    ..Default::default()
                                })
                            }
                        }
                    } else {
                        let moves = src.rook_moves(occupied);
                        if ((moves & !friendly) & blockable).intersects(dsts) {
                            return Some(MoveTrace {
                                captures,
                                takes_castle,
                                loses_castle,
                                ..MoveTrace::default()
                            })
                        }

                        if !holes_are_occ {
                            for in_sq in moves & wormholes {
                                if let Some(ray) = src.ortho_ray(in_sq) {
                                    for out_sq in wormholes {
                                        if ((ray.cast(out_sq, occupied) & !friendly) & blockable).has(dst) {
                                            return Some(MoveTrace {
                                                route: Some((in_sq, out_sq)),
                                                captures,
                                                takes_castle,
                                                loses_castle,
                                                ..Default::default()
                                            })
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Piece::Queen => {
                let blockable = crate::blockable::blockable(src, state);
                let takeable = (!friendly) | blockable;

                if wormholes.has(src) {
                    for out_sq in wormholes {
                        if ((out_sq.rook_moves(occupied) | out_sq.bishop_moves(occupied)) & takeable).has(dst) {
                            return Some(MoveTrace {
                                route: (out_sq != src).then(|| (src, out_sq)),
                                captures,
                                takes_castle,
                                ..Default::default()
                            })
                        }
                    }
                } else {
                    let moves = src.rook_moves(occupied) | src.bishop_moves(occupied);

                    if (moves & takeable).intersects(dsts) {
                        return Some(MoveTrace {
                            captures,
                            takes_castle,
                            ..Default::default()
                        })
                    }

                    if !holes_are_occ {
                        for in_sq in moves & wormholes {
                            if let Some(ray) = src.ray(in_sq) {
                                for out_sq in wormholes {
                                    if (ray.cast(out_sq, occupied) & takeable).has(dst) {
                                        return Some(MoveTrace {
                                            route: Some((in_sq, out_sq)),
                                            captures,
                                            takes_castle,
                                            ..Default::default()
                                        })
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Piece::Pawn => {
                let delta = (turn.pawn_dir(), 0);
                let pawn_rank = turn.pawn_rank();
                let blockable = crate::blockable::blockable(src, state);

                let ep_tx = state.en_passant.map(|ep_sq| BitBoard::from(ep_sq).transmit(wormholes)).unwrap_or(BitBoard(0));
                let takeable = (state.pieces.on_team(!state.turn) | ep_tx).transmit(wormholes) & blockable;
                let requires_promotion = dsts.intersects(BitBoard::from((!turn).back_rank()));

                if wormholes.has(src) {
                    let is_pawn_rank = BitBoard::from(pawn_rank).intersects(wormholes);
                    for out_sq in wormholes {
                        if (out_sq.pawn_captures(turn) & takeable).has(dst) {
                            if let Some(ep_sq) = state.en_passant && dst == ep_sq {
                                return Some(MoveTrace {
                                    route: (src != out_sq).then(|| (src, out_sq)),
                                    captures,
                                    takes_castle,
                                    is_capture_en_passant: Some(ep_sq),
                                    requires_promotion,
                                    ..Default::default()
                                })
                            } else {
                                return Some(MoveTrace {
                                    route: (src != out_sq).then(|| (src, out_sq)),
                                    captures,
                                    takes_castle,
                                    requires_promotion,
                                    ..Default::default()
                                })
                            }
                        }

                        if let Some(one) = out_sq.next(delta) && !occupied.has(one) {
                            if blockable.has(one) && one == dst {
                                return Some(MoveTrace {
                                    route: (src != out_sq).then(|| (src, out_sq)),
                                    requires_promotion,
                                    ..Default::default()
                                })
                            }

                            if let Some(two) = out_sq.next(delta) && is_pawn_rank && !occupied.has(two) {
                                if blockable.has(two) && two == dst {
                                    return Some(MoveTrace {
                                        route: (src != out_sq).then(|| (src, out_sq)),
                                        requires_promotion,
                                        allows_en_passant: Some(one),
                                        ..Default::default()
                                    })
                                }
                            }
                        }
                    }
                } else {
                    if (src.pawn_captures(turn) & takeable).intersects(dsts) {
                        if let Some(ep_sq) = state.en_passant && dst == ep_sq {
                            return Some(MoveTrace {
                                captures,
                                is_capture_en_passant: Some(ep_sq),
                                takes_castle,
                                requires_promotion,
                                ..Default::default()
                            })
                        } else {
                            return Some(MoveTrace {
                                captures,
                                takes_castle,
                                requires_promotion,
                                ..Default::default()
                            })
                        }
                    }

                    let is_pawn_rank = src.rank() == turn.pawn_rank();
                    if let Some(one) = src.next(delta) && !occupied.has(one) {
                        if blockable.has(one) && one == dst {
                            return Some(MoveTrace {
                                requires_promotion,
                                takes_castle,
                                ..Default::default()
                            })
                        } 

                        if is_pawn_rank {
                            if wormholes.has(one) {
                                for out_sq in wormholes {
                                    if let Some(two) = out_sq.next(delta) && !occupied.has(two) && blockable.has(two) {
                                        return Some(MoveTrace {
                                            route: (one != out_sq).then(|| (one, out_sq)),
                                            allows_en_passant: Some(one),
                                            requires_promotion,
                                            ..Default::default()
                                        })
                                    }
                                }
                            } else {
                                if let Some(two) = one.next(delta) && !occupied.has(two) && blockable.has(two) {
                                    return Some(MoveTrace {
                                        allows_en_passant: Some(one),
                                        requires_promotion,
                                        ..Default::default()
                                    })
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}