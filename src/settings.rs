
use chrono::{DateTime, Utc};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GameSettings {
    /// Whether or not the game is Chess960. 
    pub is_chess960: bool,

    /// The configuration of the clock, including
    /// the time the game was started, bonus time,
    /// and the total number of seconds available
    /// to each side. The time each move takes
    /// is encoded in the BoardDeltas.
    pub clock: Option<ClockSettings>,

    /// How Wormholes are spawned or placed on the board.
    pub wormhole: WormholeSettings,

    /// How castling behaves.
    pub castle: CastleSettings,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            is_chess960: false,
            castle: CastleSettings::default(),
            clock: None,
            wormhole: WormholeSettings::default(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ClockSettings {
    /// The time, in UTC, the game was started at.
    pub start: DateTime<Utc>,

    /// The bonus time, in seconds, per-move.
    pub bonus: u32,

    /// The total time available in the game, per-side.
    pub total: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct WormholeSettings {
    /// The way that Wormholes are spawned.
    pub spawn_mode: WormholeSpawnMode,   

    /// The maximum number of wormholes that can be spawned.
    pub max_count: u8, 

    /// The number of wormholes that are placed at the start
    /// of the game (according to the spawn mode). 
    pub start_count: u8,

    /// The number of halfmoves until a queued wormhole is added.
    pub hole_wait_time: u8,

    /// The number of halfmoves until a new wormhole is added
    /// to the queue. This is different from hole_wait_time so
    /// there can be a period where no holes are queued. 
    /// 
    /// If this value is 255, dynamic wormhole spawning
    /// is disabled.
    pub hole_queue_time: u8,
}

impl Default for WormholeSettings {
    fn default() -> Self {
        Self {
            spawn_mode: WormholeSpawnMode::Random,
            max_count: 0,
            start_count: 0,
            hole_wait_time: 0,
            hole_queue_time: 0,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum WormholeSpawnMode {
    /// Players manually place wormholes on the board.
    Manual,

    /// Wormhole squares are randomly assigned.
    Random,

    /// Wormhole squares will mirror placed squares.
    Mirror,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CastleSettings {
    /// The Column/File the king starts on.
    pub king_column: u8,

    /// The start column of the kingside rook.
    pub short_column: u8,

    /// The start column of the queenside rook.
    pub long_column: u8,
}

impl Default for CastleSettings {
    fn default() -> Self {
        Self {
            king_column: 4,
            short_column: 7,
            long_column: 0
        }
    }
}
