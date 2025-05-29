use crate::{board::BitBoard, state::BoardState};


/// Get a mask of squares defended by the opponent.
pub fn defense(state: &BoardState) -> BitBoard {
    let mut defense = BitBoard(0);

    if let Some(king) = state.checkable_king() {
        // The wormholes that will be available to the opponent NEXT turn.
        let wormholes = state.next_wormholes();
        // squares occupied by anything.
        let occupied = state.pieces.occupied().without(king).transmit(wormholes);
        // squares occupied by enemy pieces.
        let enemy = state.pieces.on_team(!state.turn).transmit(wormholes);

        let enemy_diag = (state.pieces.bishops | state.pieces.queens) & enemy;
        let enemy_ortho = (state.pieces.rooks | state.pieces.queens) & enemy;
        let enemy_knights = state.pieces.knights & enemy;
        let enemy_pawns = state.pieces.pawns & enemy;
        let enemy_kings = state.pieces.kings & enemy;

        // enemy diagonal sliding pieces on wormholes
        if enemy_diag.intersects(wormholes) {
            for out_sq in wormholes {
                defense |= out_sq.bishop_moves(occupied) & !wormholes;
            }
        }

        // enemy diagonal sliding pieces not on wormholes
        for enemy_sq in enemy_diag & !wormholes {
            let moves = enemy_sq.bishop_moves(occupied);
            defense |= moves;
            for in_sq in moves & wormholes & !occupied {
                if let Some(ray) = enemy_sq.diag_ray(in_sq) {
                    for out_sq in wormholes {
                        defense |= ray.cast(out_sq, occupied);
                    }
                }
            }
        }

        // enemy orthogonal sliding attacks on wormholes
        if enemy_ortho.intersects(wormholes) {
            for out_sq in wormholes {
                defense |= out_sq.rook_moves(occupied) & !wormholes;
            }
        }

        // enemy orthogonal sliding pieces not on wormholes.
        for enemy_sq in enemy_ortho & !wormholes {
            let moves = enemy_sq.rook_moves(occupied);
            defense |= moves;
            for in_sq in moves & wormholes & !occupied {
                if let Some(ray) = enemy_sq.diag_ray(in_sq) {
                    for out_sq in wormholes {
                        defense |= ray.cast(out_sq, occupied);
                    }
                }
            }
        }

        if enemy_knights.intersects(wormholes) {
            for out_sq in wormholes {
                defense |= out_sq.knight_moves() & !wormholes;
            }
        }

        for enemy_sq in enemy_knights & !wormholes {
            defense |= enemy_sq.knight_moves();
        }

        if enemy_kings.intersects(wormholes) {
            for out_sq in wormholes {
                defense |= out_sq.king_moves() & !wormholes;
            }
        }

        for enemy_sq in enemy_kings & !wormholes {
            defense |= enemy_sq.king_moves();
        }

        if enemy_pawns.intersects(wormholes) {
            defense |= (enemy_pawns & wormholes).pawn_captures(!state.turn) & !wormholes;
        }

        defense |= (enemy_pawns & !wormholes).pawn_captures(!state.turn);

        defense = defense.transmit(wormholes);
    }

    defense
}