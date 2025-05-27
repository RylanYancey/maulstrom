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

    let blockable = crate::blockable::blockable(state, sq);

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
        if holes.intersects(BitBoard::new().with_rank(team.pawn_rank())) {
            for out_sq in holes {
                let one = out_sq + (dir, 0);
                if !occupied.has(one) {
                    moves.set(one);
                    let two = out_sq + (dir, 0);
                    if !occupied.has(two) {
                        moves.set(two);
                    }
                }
            }
        } else {
            for out_sq in holes {
                let one = out_sq + (dir, 0);
                if !occupied.has(one) {
                    moves.set(one);
                }
            }
        }

        moves &= !holes;
    } else {
        let one = sq + (dir, 0);
        if !occupied.has(one) {
            moves.set(one);
            
            if sq.rank == team.pawn_rank() {
                let two = sq + (dir, 0);
                if !occupied.has(two) {
                    moves.set(two);
                }

                if holes.has(one) {
                    for out_sq in holes {
                        let one = out_sq + (dir, 0);
                        if !occupied.has(one) {
                            moves |= one;
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