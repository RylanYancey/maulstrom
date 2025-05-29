use crate::{board::BitBoard, cached::*, castle::{can_castle, Castle}, ray::*, pieces::Piece, square::Square, state::BoardState, team::Team};

pub fn compute(state: &BoardState, sq: Square) -> BitBoard {
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

    let mut ext = BitBoard::new();
    if holes.has(sq) {
        ext = holes;
    }

    if (state.pieces.get(Piece::King, team) | ext).has(sq) {
        let defense = crate::defense::defense(state);
        let mut moves = direct(sq, &KING_MOVES, holes, on_team) & !defense;

        for side in [Castle::Long, Castle::Short] {
            if can_castle(side, team, state.castle, defense, occupied, sq) {
                moves |= state.castle.rook_start(side, team);
                moves |= state.castle.king_target(side, team);
            }
        }
        
        return moves
    }

    let blockable = BitBoard(!0); // crate::blockable::blockable(sq, state, relevant);

    if (state.pieces.get(Piece::Pawn, team) | ext).has(sq) {
        return blockable & pawn(team, sq, holes, on_team, occupied, state.en_passant);
    }

    if (state.pieces.get(Piece::Rook, team) | ext).has(sq) {
        let mut moves = BitBoard::new();

        // player may be able to castle by moving rook to king
        for side in [Castle::Long, Castle::Short] {
            if sq == state.castle.rook_start(side, team) {
                if let Some(king) = state.pieces.get(Piece::King, team).first() {
                    let defense = crate::defense::defense(state);
                    if can_castle(side, team, state.castle, defense, occupied, king) {
                        moves |= king
                    }
                }
            }
        }

        return blockable & (
            moves |
            ray(occupied, pos_zero, sq, holes, on_team) |
            ray(occupied, neg_zero, sq, holes, on_team) |
            ray(occupied, zero_pos, sq, holes, on_team) |
            ray(occupied, zero_neg, sq, holes, on_team)
        )
    }

    if (state.pieces.get(Piece::Knight, team) | ext).has(sq) {
        return direct(sq, &KNIGHT_MOVES, holes, on_team) & blockable;
    }

    if (state.pieces.get(Piece::Queen, team) | ext).has(sq) {
        return blockable & (
            ray(occupied, pos_pos, sq, holes, on_team) |
            ray(occupied, neg_neg, sq, holes, on_team) |
            ray(occupied, pos_neg, sq, holes, on_team) |
            ray(occupied, neg_pos, sq, holes, on_team) |
            ray(occupied, pos_zero, sq, holes, on_team) |
            ray(occupied, neg_zero, sq, holes, on_team) |
            ray(occupied, zero_pos, sq, holes, on_team) |
            ray(occupied, zero_neg, sq, holes, on_team)
        )
    }

    if (state.pieces.get(Piece::Bishop, team) | ext).has(sq) {
        return blockable & (
            ray(occupied, pos_pos, sq, holes, on_team) |
            ray(occupied, neg_neg, sq, holes, on_team) |
            ray(occupied, pos_neg, sq, holes, on_team) |
            ray(occupied, neg_pos, sq, holes, on_team)
        )
    }

    return BitBoard(0)
}

fn pawn(
    team: Team, 
    sq: Square, 
    holes: BitBoard,
    on_team: BitBoard, 
    occupied: BitBoard,
    en_passant: Option<Square>,
) -> BitBoard {
    let mut captures = match team {
        Team::White => direct(sq, &WHITE_PAWN_ATTACKS, holes, on_team),
        Team::Black => direct(sq, &BLACK_PAWN_ATTACKS, holes, on_team),
    };

    if let Some(en_passant) = en_passant {
        captures &= occupied | en_passant;
    } else {
        captures &= occupied;
    }

    let mut moves = BitBoard::new();
    let dir = team.pawn_dir();

    if holes.has(sq) {
        if holes.intersects(BitBoard::new().with_rank_u8(team.pawn_rank_u8())) {
            for out_sq in holes {
                if let Some(one) = out_sq.next((dir, 0)) && !occupied.has(one) {
                    moves.set(one);
                    if let Some(two) = one.next((dir, 0)) && !occupied.has(two) {
                        moves.set(two);
                    }
                }
            }
        } else {
            for out_sq in holes {
                if let Some(one) = out_sq.next((dir, 0)) && !occupied.has(one) {
                    moves.set(one);
                }
            }
        }

        moves &= !holes;
    } else {
        if let Some(one) = sq.next((dir, 0)) && !occupied.has(one) {
            moves.set(one);
            if sq.rank() == team.pawn_rank() {
                if let Some(two) = one.next((dir, 0)) && !occupied.has(two) {
                    moves.set(two);
                }

                if holes.has(one) {
                    for out_sq in holes {
                        if let Some(one) = out_sq.next((dir, 0)) && !occupied.has(one) {
                            moves.set(one);
                        }
                    }
                }
            }
        }
    }

    moves | captures
}

fn direct(
    square: Square,
    table: &[u64; 64],
    holes: BitBoard,
    on_team: BitBoard,
) -> BitBoard {
    if holes.has(square) {
        holes.into_iter().fold(BitBoard::new(), |mask, out_sq| {
            mask | BitBoard(table[out_sq.to_index()])
        }) & !(on_team | holes)
    } else {
        let mvs = BitBoard(table[square.to_index()]) & !on_team;
        if holes.intersects(mvs) {
            mvs | holes
        } else {
            mvs
        }
    }
}

fn ray(
    occupied: BitBoard,
    delta: fn(Square, BitBoard) -> BitBoard,
    piece: Square,
    holes: BitBoard,
    on_team: BitBoard,
) -> BitBoard {
    if holes.has(piece) {
        holes.into_iter().fold(BitBoard::new(), |mask, out_sq| {
            mask | delta(out_sq, occupied)
        }) & !(on_team | holes)
    } else {
        let mut ray = delta(piece, occupied);

        if ray.intersects(holes) && !occupied.intersects(holes) {
            for out_sq in holes & !ray {
                ray |= delta(out_sq, occupied);
            }
        }

        ray & !(on_team | piece)
    }
}

pub fn compute2(state: &BoardState, sq: Square, defense: Option<BitBoard>, relevant: BitBoard) -> BitBoard {
    let wormholes = state.wormholes;
    let mut moves = BitBoard(0);
    let occupied = state.pieces.occupied().transmit(wormholes);
    let turn = state.turn;
    let friendly = state.pieces.on_team(turn);

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
                    for in_sq in (moves & !occupied) & wormholes {
                        if let Some(ray) = sq.ray(in_sq) {
                            for out_sq in wormholes {
                                moves |= ray.cast(out_sq, occupied);
                            }
                        }
                    }
                }
                moves &= (!friendly) | crate::blockable::blockable(sq, state, relevant);
            },
            Piece::Bishop => {
                if wormholes.has(sq) {
                    for out_sq in wormholes {
                        moves |= out_sq.bishop_moves(occupied) & !wormholes;                    
                    }
                } else {
                    moves |= sq.bishop_moves(occupied);
                    for in_sq in (moves & !occupied) & wormholes {
                        if let Some(ray) = sq.diag_ray(in_sq) {
                            for out_sq in wormholes {
                                moves |= ray.cast(out_sq, occupied);
                            }
                        }
                    }
                }
                moves &= (!friendly) | crate::blockable::blockable(sq, state, relevant);
            },
            Piece::Knight => {
                if wormholes.has(sq) {
                    for out_sq in wormholes {
                        moves |= out_sq.knight_moves() & !wormholes;
                    } 
                } else {
                    moves |= sq.knight_moves();
                }
                moves &= (!friendly) | crate::blockable::blockable(sq, state, relevant);
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
                moves |= (captures & (ep_tx | enemy)) & crate::blockable::blockable(sq, state, relevant);
            },
            Piece::Rook => {
                if wormholes.has(sq) {
                    for out_sq in wormholes {
                        moves |= out_sq.rook_moves(occupied) & !wormholes;
                    }
                } else {
                    moves |= sq.bishop_moves(occupied);
                    for in_sq in (moves & !occupied) & wormholes {
                        if let Some(ray) = sq.ortho_ray(in_sq) {
                            for out_sq in wormholes {
                                moves |= ray.cast(out_sq, occupied);
                            }
                        }
                    }
                }

                let blockable = crate::blockable::blockable(sq, state, relevant);
                moves &= (!friendly) | blockable;

                if blockable != BitBoard(!0) {
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