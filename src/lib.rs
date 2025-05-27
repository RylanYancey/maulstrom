
pub mod board;
pub mod square;
pub mod cached;
pub mod defense;
pub mod blockable;
pub mod compute;
pub mod settings;
pub mod team;
pub mod state;
pub mod pieces;
pub mod castle;
pub mod ray;
pub mod init;
pub mod game;
pub mod trace;
pub mod end;
pub mod rng;
pub mod delta;

pub mod prelude {
    pub use crate::{
        game::{ChessGame, Cursor},
        board::{BitBoard, BitBoardIndices, BitBoardIter},
        castle::{CastleRights, Castle, CastleSettings},
        pieces::{Piece, Pieces},
        square::Square,
        settings::{GameSettings, ClockSettings, WormholeSettings, WormholeSpawnMode},
        trace::MoveTrace,
        end::EndCondition,
        team::Team,
    };
}