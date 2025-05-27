
use crate::{board::BitBoard, castle::{Castle, CastleRights}, pieces::{Piece, Pieces}, rng::WyRand, state::BoardState, team::Team};

pub fn init_chess960(rng: &mut WyRand) -> BoardState {
    let mut indices = [0, 1, 2, 3, 4, 5, 6, 7];
    rng.shuffle(&mut indices);

    // bishops must be on opposite color squares.
    for i in 0..7 {
        if indices[5] & 1 == indices[6] & 1 {
            indices.swap(i, 5)
        } else {
            break;
        }
    }

    // king must be to the right of the long rook.
    if indices[0] > indices[1] {
        indices.swap(0, 1);
    }

    // king must be to the left of the short rook.
    if indices[1] > indices[2] {
        indices.swap(1, 2);
    }

    let mut pieces = Pieces::just_pawns();
    pieces.setup_from_file(Piece::Rook, indices[0]);
    pieces.setup_from_file(Piece::King, indices[1]);
    pieces.setup_from_file(Piece::Rook, indices[2]);
    pieces.setup_from_file(Piece::Queen, indices[3]);
    pieces.setup_from_file(Piece::Knight, indices[4]);
    pieces.setup_from_file(Piece::Knight, indices[5]);
    pieces.setup_from_file(Piece::Bishop, indices[6]);
    pieces.setup_from_file(Piece::Bishop, indices[7]);

    let mut castle = CastleRights::default();
    castle.set_rook(Castle::Long, indices[0]);
    castle.set_rook(Castle::Short, indices[2]);
    castle.set_king(indices[1]);

    BoardState {
        en_passant: None,
        next_hole: None,
        hole_in_1: false,
        wormholes: BitBoard::new(),
        fullmoves: 1,
        halfmoves: 0,
        pieces,
        castle,
        turn: Team::White,
    }
}