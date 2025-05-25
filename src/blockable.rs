use crate::{board::BitBoard, cached::*, delta::*, pieces::Piece, square::Square, state::BoardState, team::Team};

/// Compute the squares a piece could move to to maintain a pin, block check, or capture a checking piece.
/// This can be used to filter out moves. If there is no check or pins, then the returned mask with be u64::MAX.
pub fn blockable(state: &BoardState, pc: Square) -> BitBoard {
    let Some(king) = state.pieces.get(Piece::King, state.turn).first() else { return BitBoard(!0u64) };
    let team = !state.turn;
    let holes = state.wormholes;
    let mut occupied = state.pieces.occupied().without(king);
    if occupied.intersects(holes) {
        occupied |= holes;
    }

    let mut blockable = BitBoard(!0u64);
    direct(&mut blockable, &KNIGHT_MOVES, state.pieces.get(Piece::Knight, team), holes, king);
    direct(&mut blockable, &KING_MOVES, state.pieces.get(Piece::King, team), holes, king);
    let diag = state.pieces.all(&[Piece::Queen, Piece::Bishop], team);
    ray(&mut blockable, occupied, diag, pos_pos, pc, holes, king);
    ray(&mut blockable, occupied, diag, neg_neg, pc, holes, king);
    ray(&mut blockable, occupied, diag, pos_neg, pc, holes, king);
    ray(&mut blockable, occupied, diag, neg_pos, pc, holes, king);
    let ortho = state.pieces.all(&[Piece::Queen, Piece::Rook], team);
    ray(&mut blockable, occupied, ortho, pos_zero, pc, holes, king);
    ray(&mut blockable, occupied, ortho, neg_zero, pc, holes, king);
    ray(&mut blockable, occupied, ortho, zero_pos, pc, holes, king);
    ray(&mut blockable, occupied, ortho, zero_neg, pc, holes, king);
    match state.turn {
        Team::White => direct(&mut blockable, &WHITE_PAWN_ATTACKS, state.pieces.get(Piece::Pawn, team), holes, king),
        Team::Black => direct(&mut blockable, &BLACK_PAWN_ATTACKS, state.pieces.get(Piece::Pawn, team), holes, king)
    }

    blockable
}

fn direct(
    blockable: &mut BitBoard,
    table: &[u64; 64],
    relevant: BitBoard,
    holes: BitBoard,
    king: Square,
) {
    if holes.has(king) {
        for out_sq in holes {
            let rel = relevant & BitBoard(table[out_sq.to_index()]);
            match rel.count() {
                0 => {},
                1 => *blockable &= rel,
                _ => *blockable = BitBoard(0),
            }
        }
    } else {
        let mvs = BitBoard(table[king.to_index()]);
        let rel = mvs & relevant;

        match rel.count() {
            0 => {},
            1 => *blockable &= rel,
            _ => *blockable = BitBoard(0),
        }

        if mvs.intersects(holes) && relevant.intersects(holes) {
            *blockable &= holes;
        }
    }
}

fn ray(
    blockable: &mut BitBoard, 
    occupied: BitBoard,
    mut relevant: BitBoard,
    delta: fn(Square, BitBoard) -> BitBoard,
    piece: Square, 
    holes: BitBoard,
    king: Square, 
) {
    let can_block = occupied & !relevant;
    if holes.intersects(relevant) { 
        relevant |= holes 
    }

    if holes.has(king) {
        for out_sq in holes {
            let ray = delta(out_sq, relevant);

            if ray.intersects(relevant) {
                let blocking = (can_block & ray).count();
                if blocking == 0 || (blocking == 1 && ray.has(piece)) {
                    *blockable &= ray;
                }
            }
        }
    } else {
        // the ray until the first relevant attacking piece or edge of board.
        let ray = delta(king, relevant);    

        if ray.intersects(relevant) {
            let blocking = (can_block & ray).count();
            // if there is a relevant attacking piece on the ray, and
            // there is nothing blocking the ray OR only the piece is 
            // blocking the ray, then add the ray to blockable.
            if blocking == 0 || (blocking == 1 && ray.has(piece)) {
                *blockable &= ray;
            }
        }

        if holes.intersects(ray & !relevant) {
            // squares between the king and the first wormhole on this ray.
            let to_in_sq = delta(king, holes);
            let blocking = (can_block & to_in_sq).count();

            if blocking == 0 || (blocking == 1 && to_in_sq.has(piece)) {
                for out_sq in holes & !to_in_sq {
                    let from_out_sq = delta(out_sq, relevant);
    
                    if ray.intersects(relevant) {
                        let blocking = blocking + (can_block & from_out_sq).count();
                        let squares = to_in_sq | from_out_sq;
    
                        if blocking == 0 || (blocking == 1 && squares.has(piece)) {
                            *blockable &= squares;
                        }
                    }
                }
            }
        } 
    }
}
