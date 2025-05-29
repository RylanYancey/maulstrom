
pub mod slide_table;
pub mod blockable;
pub mod settings;
pub mod defense;
pub mod compute;
pub mod castle;
pub mod pieces;
pub mod square;
pub mod cached;
pub mod board;
pub mod state;
pub mod trace;
pub mod delta;
pub mod magic;
pub mod team;
pub mod init;
pub mod game;
pub mod ray;
pub mod end;
pub mod rng;

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