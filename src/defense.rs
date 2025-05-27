use crate::{board::BitBoard, cached::*, ray::*, pieces::Piece, square::Square, state::BoardState, team::Team};


pub fn defense(state: &BoardState) -> BitBoard {
    let team = !state.turn;
    let holes = state.wormholes;
    let king = state.pieces.get(Piece::King, state.turn);
    let mut occupied = state.pieces.occupied() & !king;
    if holes.intersects(occupied) {
        occupied |= holes;
    }

    let mut defense = BitBoard::new()
        | pawn_defense(team, state.pieces.get(Piece::Pawn, team), holes)
        | king_defense(state.pieces.get(Piece::King, team), holes)
        | knight_defense(state.pieces.get(Piece::Knight, team), holes)
        | bishop_defense(state.pieces.all(&[Piece::Queen, Piece::Bishop], team), holes, occupied)
        | rook_defense(state.pieces.all(&[Piece::Queen, Piece::Rook], team), holes, occupied);

    if holes.intersects(defense) {
        defense |= holes;
    }

    defense
}

/// Compute squares defended by pawns.
fn pawn_defense(team: Team, pieces: BitBoard, holes: BitBoard) -> BitBoard {
    fn compute(team: Team, p: u64) -> u64 {
        let pr = p & !0x8080808080808080;
        let pl = p & !0x0101010101010101;
        match team {
            Team::White => (pl << 7) | (pr << 9),
            Team::Black => (pl >> 9) | (pr >> 7)
        }
    }

    if pieces.intersects(holes) {
        BitBoard(
            compute(team, (pieces & !holes).0) | 
            compute(team, (pieces & holes).0) & !holes.0
        )
    } else {
        BitBoard(compute(team, pieces.0))
    }
}

/// Compute squares defended by knights.
fn knight_defense(pieces: BitBoard, holes: BitBoard) -> BitBoard {
    fn compute(pcs: BitBoard) -> BitBoard {
        pcs.indices().fold(BitBoard::new(), |mask, i| {
            mask | KNIGHT_MOVES[i as usize]
        })
    }

    if pieces.intersects(holes) {
        compute(pieces & !holes) | (compute(holes) & !holes)
    } else {
        compute(pieces)        
    }
}

/// Compute squares defended by kings.
fn king_defense(pieces: BitBoard, holes: BitBoard) -> BitBoard {
    fn compute(pcs: BitBoard) -> BitBoard {
        pcs.indices().fold(BitBoard::new(), |mask, i| {
            mask | KING_MOVES[i as usize]
        })
    }

    if pieces.intersects(holes) {
        compute(pieces & !holes) | (compute(holes) & !holes)
    } else {
        compute(pieces)        
    }
}

/// Compute squares defended by orthogonal sliders.
fn rook_defense(pieces: BitBoard, holes: BitBoard, occupied: BitBoard) -> BitBoard {
    pieces.into_iter().fold(BitBoard::new(), |mask, src| {
        mask | 
            ray(src, pos_zero, occupied, holes) |
            ray(src, neg_zero, occupied, holes) |
            ray(src, zero_pos, occupied, holes) |
            ray(src, zero_neg, occupied, holes)
    })
}

/// Compute squares defended by diagonal sliders.
fn bishop_defense(pieces: BitBoard, holes: BitBoard, occupied: BitBoard) -> BitBoard {
    pieces.into_iter().fold(BitBoard::new(), |mask, src| {
        mask |
            ray(src, pos_pos, occupied, holes) |
            ray(src, neg_neg, occupied, holes) |
            ray(src, pos_neg, occupied, holes) |
            ray(src, neg_pos, occupied, holes)
    })
}

/// Compute squares for a sliding piece on some ray.
fn ray(
    src: Square, 
    delta: fn(Square, BitBoard) -> BitBoard,
    occupied: BitBoard,
    holes: BitBoard,
) -> BitBoard {
    if holes.has(src) {
        holes.into_iter().fold(BitBoard::new(), |mask, out_sq| {
            mask | delta(out_sq, occupied.without(out_sq))
        }) & !holes
    } else {
        let ray = delta(src, occupied.without(src));
    
        if holes.intersects(ray) {
            (holes & !ray).into_iter().fold(ray, |ray, out_sq| {
                ray | delta(out_sq, occupied)
            }).without(src)
        } else {
            ray
        }
    }
}