use crate::{board::BitBoard, cached::*, ray::*, pieces::Piece, square::Square, state::BoardState, team::Team};

/// Compute the squares a piece could move to to maintain a pin, block check, or capture a checking piece.
/// This can be used to filter out moves. If there is no check or pins, then the returned mask with be u64::MAX.
pub fn blockable(sq: Square, state: &BoardState, relevant: BitBoard) -> BitBoard {
    let mut blockable = BitBoard(!0u64);
    if let Some(king) = state.checkable_king() {
        // The wormholes that will be available to the opponent NEXT turn.
        let wormholes = state.next_wormholes();
        // squares occupied by anything.
        let occupied = state.pieces.occupied().without(king).transmit(wormholes);
        // squares occupied by enemy pieces.
        let enemy = (state.pieces.on_team(!state.turn) & relevant).transmit(wormholes);

        let enemy_diag = (state.pieces.bishops | state.pieces.queens) & enemy;
        let enemy_ortho = (state.pieces.rooks | state.pieces.queens) & enemy;
        let enemy_knights = state.pieces.knights & enemy;
        let enemy_pawns = state.pieces.pawns & enemy;
        let enemy_kings = state.pieces.kings & enemy;

        if wormholes.has(king) {
            // the king is on a wormhole

            for out_sq in wormholes {
                // compute pins and checks by diagonally sliding pieces.
                let diag = out_sq.bishop_moves(enemy_diag);
                for enemy_sq in enemy_diag & diag {
                    let between = out_sq.between(enemy_sq);
                    let cnt = (between & occupied).count();
                    if cnt == 0 || (cnt == 1 && between.has(sq)) {
                        blockable &= (between | enemy_sq).transmit(wormholes)
                    }
                }

                // compute pins and checks by orthogonally sliding pieces.
                let ortho = out_sq.rook_moves(enemy_ortho);
                for enemy_sq in enemy_ortho & ortho {
                    let between = out_sq.between(enemy_sq);
                    let cnt = (between & occupied).count();
                    if cnt == 0 || (cnt == 1 && between.has(sq)) {
                        blockable &= (between | enemy_sq).transmit(wormholes)
                    }
                }

                // compute direct checks by pawns, kings, and knights.
                for enemy_sq in 
                    (out_sq.knight_moves() & enemy_knights) |
                    (out_sq.king_moves() & enemy_kings) | 
                    (out_sq.pawn_captures(state.turn) & enemy_pawns) 
                {
                    blockable &= enemy_sq;
                }
            }
        } else {
            let diag = king.bishop_moves(enemy_diag);
            for enemy_sq in diag & enemy_diag {
                let btw = king.between(enemy_sq);
                let cnt = ((btw & !enemy_diag) & occupied).count();
                if cnt == 0 || (cnt == 1 && btw.has(sq)) {
                    blockable &= (btw | enemy_sq).transmit(wormholes);
                }
            }

            for in_sq in diag & wormholes {
                let btw1 = king.between(in_sq);
                let cnt1 = (btw1 & occupied).count();
                if cnt1 == 0 || (cnt1 == 1 && btw1.has(sq)) {
                    if let Some(ray) = king.diag_ray(in_sq) {
                        for out_sq in wormholes {
                            if let Some(btw2) = ray.cast_if_hit(out_sq, enemy_diag) {
                                let path = btw1 | btw2;
                                let cnt2 = ((path &! enemy_diag) & occupied).count();
                                if cnt2 == 0 || (cnt2 == 1 && path.has(sq)) {
                                    blockable &= path.transmit(wormholes);
                                }
                            }
                        }
                    }
                }
            }

            let ortho = king.rook_moves(enemy_ortho);
            for enemy_sq in ortho & enemy_diag {
                let btw = king.between(enemy_sq);
                let cnt = ((btw & !enemy_ortho) & occupied).count();
                if cnt == 0 || (cnt == 1 && btw.has(sq)) {
                    blockable &= (btw | enemy_sq).transmit(wormholes);
                }
            }

            for in_sq in ortho & wormholes {
                let btw1 = king.between(in_sq);
                let cnt1 = (btw1 & occupied).count();
                if cnt1 == 0 || (cnt1 == 1 && btw1.has(sq)) {
                    if let Some(ray) = king.ortho_ray(in_sq) {
                        for out_sq in wormholes {
                            if let Some(btw2) = ray.cast_if_hit(out_sq, enemy_ortho) {
                                let path = btw1 | btw2;
                                let cnt2 = ((path & !enemy_ortho) & occupied).count();
                                if cnt2 == 0 || (cnt2 == 1 && path.has(sq)) {
                                    blockable &= path.transmit(wormholes);
                                }
                            }
                        }
                    }
                }
            }

            for enemy_sq in 
                (king.knight_moves().transmit(wormholes) & enemy_knights) |
                (king.king_moves().transmit(wormholes) & enemy_kings) | 
                (king.pawn_captures(state.turn).transmit(wormholes) & enemy_pawns) 
            {
                blockable &= enemy_sq;
            }
        }
    } 

    blockable
}
