use crate::{board::BitBoard, cached::*, castle::{can_castle, Castle}, ray::*, pieces::Piece, square::Square, state::BoardState, team::Team};

pub fn compute(state: &BoardState, sq: Square, defense: Option<BitBoard>) -> BitBoard {
    let wormholes = state.wormholes;
    let mut moves = BitBoard(0);
    let occupied = state.pieces.occupied().transmit(wormholes);
    let turn = state.turn;
    let friendly = state.pieces.on_team(turn);
    let holes_are_occ = occupied.intersects(wormholes);

    if let Some(pc) = state.pieces.piece_at_or_on_hole(sq, wormholes) {
        match pc {
            Piece::King => {
                let defense = defense.unwrap_or_else(|| crate::defense::defense(state));
            
                if wormholes.has(sq) {
                    for out_sq in wormholes {
                        moves |= out_sq.king_moves() & !wormholes;
                    }
                } else {
                    moves |= sq.king_moves();
                }
    
                // cannot capture friendly or defended squares as king.
                moves &= !(friendly | defense);
    
                for side in [Castle::Long, Castle::Short] {
                    if can_castle(side, turn, state.castle, defense, occupied, sq) {
                        moves |= state.castle.rook_start(side, turn);
                        moves |= state.castle.king_target(side, turn);
                    }
                }
            },
            Piece::Queen => {
                if wormholes.has(sq) {
                    for out_sq in wormholes {
                        moves |= out_sq.bishop_moves(occupied) | out_sq.rook_moves(occupied);
                    }
                    moves &= !wormholes;
                } else {
                    moves = sq.bishop_moves(occupied) | sq.rook_moves(occupied);
                    if !holes_are_occ {
                        for in_sq in (moves & !occupied) & wormholes {
                            if let Some(ray) = sq.ray(in_sq) {
                                for out_sq in wormholes {
                                    moves |= ray.cast(out_sq, occupied);
                                }
                            }
                        }
                    }
                }
                moves &= (!friendly) | crate::blockable::blockable(sq, state);
            },
            Piece::Bishop => {
                if wormholes.has(sq) {
                    for out_sq in wormholes {
                        moves |= out_sq.bishop_moves(occupied) & !wormholes;                    
                    }
                } else {
                    moves |= sq.bishop_moves(occupied);
                    if !holes_are_occ {
                        for in_sq in (moves & !occupied) & wormholes {
                            if let Some(ray) = sq.diag_ray(in_sq) {
                                for out_sq in wormholes {
                                    moves |= ray.cast(out_sq, occupied);
                                }
                            }
                        }
                    }
                }
                moves &= (!friendly) | crate::blockable::blockable(sq, state);
            },
            Piece::Knight => {
                if wormholes.has(sq) {
                    for out_sq in wormholes {
                        moves |= out_sq.knight_moves() & !wormholes;
                    } 
                } else {
                    moves |= sq.knight_moves();
                }
                moves &= (!friendly) | crate::blockable::blockable(sq, state);
            },
            Piece::Pawn => {
                let mut captures = BitBoard(0);
                let pawn_rank = BitBoard::new().with_rank_u8(turn.pawn_rank_u8());
                let delta = (turn.pawn_dir(), 0);

                if wormholes.has(sq) {
                    let is_pawn_rank = pawn_rank.intersects(wormholes);
                    for out_sq in wormholes {
                        captures |= out_sq.pawn_captures(turn);
                        if let Some(one) = out_sq.next(delta) && !occupied.has(one) {
                            moves |= one;
                            if let Some(two) = one.next(delta) && is_pawn_rank {
                                moves |= two;
                            }
                        }
                    }
                } else {
                    let is_pawn_rank = pawn_rank.has(sq);
                    captures |= sq.pawn_captures(turn);
                    if let Some(one) = sq.next(delta) && !occupied.has(one) {
                        moves |= one;
                        if is_pawn_rank {
                            if wormholes.has(one) {
                                for out_sq in wormholes {
                                    if let Some(two) = out_sq.next(delta) && !occupied.has(two) {
                                        moves |= two;
                                    }
                                }
                            } else {
                                if let Some(two) = one.next(delta) && !occupied.has(two) {
                                    moves |= two;
                                }
                            }
                        }
                    }
                }

                let ep_tx = state.en_passant.map(|ep_sq| BitBoard::from(ep_sq).transmit(wormholes)).unwrap_or(BitBoard(0));
                let enemy = state.pieces.on_team(!state.turn).transmit(wormholes);
                moves |= (captures & (ep_tx | enemy)) & crate::blockable::blockable(sq, state);
            },
            Piece::Rook => {
                if wormholes.has(sq) {
                    for out_sq in wormholes {
                        moves |= out_sq.rook_moves(occupied) & !wormholes;
                    }
                } else {
                    moves |= sq.rook_moves(occupied);
                    if !holes_are_occ {
                        for in_sq in (moves & !occupied) & wormholes {
                            if let Some(ray) = sq.ortho_ray(in_sq) {
                                for out_sq in wormholes {
                                    moves |= ray.cast(out_sq, occupied);
                                }
                            }
                        }
                    }
                }

                let blockable = crate::blockable::blockable(sq, state);
                moves &= (!friendly) | blockable;

                if blockable == BitBoard(!0) {
                    for side in [Castle::Short, Castle::Long] {
                        if sq == state.castle.rook_start(side, turn) {
                            for king in state.pieces.get(Piece::King, turn) {
                                let defense = defense.unwrap_or_else(|| crate::defense::defense(state));
                                if can_castle(side, turn, state.castle, defense, occupied, king) {
                                    moves |= king;
                                }
                            }
                        }
                    }
                }
            },
        }
    }

    moves.transmit(wormholes)
}