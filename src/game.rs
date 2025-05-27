use chrono::Duration;

use crate::{board::BitBoard, castle::{Castle, CastleRights}, delta::BoardDelta, end::EndCondition, pieces::Piece, rng::WyRand, settings::GameSettings, square::Square, state::BoardState, trace::MoveTrace};

#[derive(Clone)]
pub struct ChessGame {
    /// The state of the board on the first move.
    pub start: BoardState,

    /// The position currently being viewed.
    pub cursor: Cursor,

    /// The changes that occur at each move in the game.
    pub deltas: Vec<BoardDelta>,

    /// Settings that control wormhole and clock behavior.
    pub settings: GameSettings,

    /// A Unique identifier for this game.
    /// If Branch=Some, then this is the id
    /// of the branch.
    pub game_id: u64,

    /// Whether or not this ChessGame instance 
    /// is a branch of an existing game. If so,
    /// this will be Some(index_of_position).
    pub is_branch: Option<Branch>,

    /// RNG seed used to select which squares
    /// should have wormholes, and/or which square
    /// wormholes will be added to if on a mode that
    /// supports dynamically spawning wormholes.
    /// 
    /// Used to select Chess960 position as well.
    pub seed: u64,

    /// Whether the last position in the game is checkmate,
    /// stalemate, or any other end condition.
    pub end: Option<EndCondition>,
}

impl ChessGame {
    pub fn init(settings: GameSettings) -> Self {
        let seed = crate::rng::entropy();
        let mut rng = WyRand { seed };

        let start = if settings.is_chess960 {
            crate::init::init_chess960(&mut rng)
        } else {
            BoardState::default()
        };

        let cursor = if let Some(clock) = settings.clock {
            Cursor {
                state: start,
                index: 0,
                white_time: clock.total,
                black_time: clock.total,
                clock_is_ticking: true,
            }
        } else {
            Cursor::new(start)
        };

        Self {
            start,
            cursor,
            deltas: Vec::new(),
            settings,
            game_id: crate::rng::entropy(),
            is_branch: None,
            seed,
            end: None,
        }
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_is_last(&self) -> bool {
        self.cursor.index == self.deltas.len()
    }

    /// Get the delta for the move that was played in the cursor position.
    /// If this is the last move in the game, None is returned.
    pub fn get_next_delta(&self) -> Option<BoardDelta> {
        self.deltas.get(self.cursor.index).copied()
    }

    /// Get the delta for the move that was played to reach this cursor position.
    pub fn get_prev_delta(&self) -> Option<BoardDelta> {
        if self.cursor.index == 0 {
            self.is_branch.map(|br| br.delta)
        } else {
            self.deltas.get(self.cursor.index - 1).copied()
        }
    }

    pub fn branch(&mut self, delta: BoardDelta) -> ChessGame {
        let next = self.cursor.state.next(delta);

        let halfmoves = if let Some(branch) = self.is_branch {
            branch.src_halfmoves as usize + self.cursor.index
        } else {
            self.cursor.index
        };

        Self {
            start: next,
            cursor: Cursor::new(next),
            deltas: Vec::new(),
            settings: self.settings,
            game_id: crate::rng::entropy(),
            is_branch: Some(
                Branch {
                    parent_id: self.game_id,
                    src_index: self.cursor.index,
                    delta,
                    src_halfmoves: halfmoves as u16
                }
            ),
            seed: self.seed,
            end: None, // todo: figure this out
        }
    }

    pub fn play(
        &mut self, 
        src: Square, 
        dst: Square, 
        promote: Option<Piece>
    ) -> Result<PlaySuccess, PlayError> {
        if self.cursor_is_last()  {
            if let Some(condition) = self.end {
                return Err(PlayError::GameEnded(condition))
            } 
        } 

        if let Some(trace) = self.cursor.state.trace(src, dst) {
            let mut delta = BoardDelta::default();
            if trace.requires_promotion {
                if promote.is_none_or(|pc| matches!(pc, Piece::Pawn | Piece::King)) {
                    return Err(PlayError::RequiresPromotion)
                } else {
                    delta.set_promote_pc(promote.unwrap());
                }
            }

            let prev = self.cursor.state;
            let mut castle = prev.castle;

            delta.set_prev_halfmoves(prev.halfmoves);

            if let Some(side) = trace.is_castle {
                castle.lose(Castle::Short, prev.turn);
                castle.lose(Castle::Long, prev.turn);
                delta.set_src_sq(castle.king_start(prev.turn));
                delta.set_dst_sq(castle.rook_target(side, prev.turn));
                delta.set_is_castle(side);
            } else {
                delta.set_src_sq(src);
                delta.set_dst_sq(dst);    

                if let Some(side) = trace.loses_castle {
                    castle.lose(side, prev.turn);
                }
    
                if let Some(side) = trace.takes_castle {
                    castle.lose(side, !prev.turn);
                }

                if let Some(capture) = trace.captures {
                    delta.set_capture_pc(capture);
                }
    
                if let Some(ep_sq) = prev.en_passant {
                    delta.set_prev_ep_sq(ep_sq);
                }
    
                if let Some(_) = trace.allows_en_passant {
                    delta.set_is_double_push();
                } else {
                    if let Some(ep_capture_sq) = trace.is_capture_en_passant {
                        delta.set_ep_capture_sq(ep_capture_sq);
                    }
                }
            }

            if trace.is_king_move {
                castle.lose(Castle::Long, prev.turn);
                castle.lose(Castle::Short, prev.turn);
            } 

            delta.set_castle_deltas(prev.castle.rights, castle.rights);
            delta.set_prev_halfmoves(prev.halfmoves);

            // if the cursor is not last, the move must either be 
            // equal to the existing move (advancement) or create
            // a branch if different. 
            if !self.cursor_is_last() {
                if self.get_next_delta().is_some_and(|del| {
                    del.get_src_sq() == delta.get_src_sq() && 
                    del.get_dst_sq() == delta.get_dst_sq()
                }) {
                    return Ok(
                        PlaySuccess {
                            branch: Some(self.branch(delta)),
                            delta,
                            trace,
                        }
                    )
                } 
            } else {
                self.deltas.push(delta);

                // todo: tick the clock
            }

            self.cursor.index += 1;
            self.cursor.state = self.cursor.state.next(delta);
            Ok(
                PlaySuccess {
                    branch: None,
                    delta,
                    trace
                }
            )
        } else {
            Err(PlayError::InvalidMove)
        }
    }

    pub fn next(&mut self) -> Option<&Cursor> {
        self.get_next_delta().map(|delta| {
            self.cursor.state = self.cursor.state.next(delta);
            self.cursor.index += 1;
            &self.cursor
        })
    }

    pub fn prev(&mut self) -> Option<&Cursor> {
        if !self.cursor.index == 0 {
            self.get_prev_delta().map(|delta| {
                self.cursor.state = self.cursor.state.prev(delta);
                self.cursor.index -= 1;
                &self.cursor
            })
        } else {
            None
        }
    }
}

impl Default for ChessGame {
    fn default() -> Self {
        Self {
            start: BoardState::default(),
            cursor: Cursor::default(),
            deltas: Vec::new(),
            settings: GameSettings::default(),
            is_branch: None,
            game_id: 0,
            seed: 0,
            end: None,
        }
    }
}

pub struct PlaySuccess {
    /// If you try to play a move when the cursor is not at
    /// the last position, and the move is valid, but not the
    /// same as the move played at that position, then a new board
    /// is created for the new game.
    pub branch: Option<ChessGame>,

    /// Compressed board changes.
    pub delta: BoardDelta,

    /// Includes more descriptive information about the board changes,
    /// such as the route taken to arrive at the destination square.
    /// Use this to determine which wormholes to travel through,
    /// if any, when animating moves.
    pub trace: MoveTrace,
}

pub enum PlayError {
    /// The current position cannot be
    /// advanced because an end condition
    /// has been reached.
    GameEnded(EndCondition),

    /// The move is illegal and cannot occur.
    InvalidMove,

    /// The move cannot be performed unless the
    /// "promote" argument is set to a valid piece.
    RequiresPromotion,
}

#[derive(Copy, Clone)]
pub struct Cursor {
    /// The state at the cursor.
    pub state: BoardState,

    /// The halfmove index of the cursor position, where the start position=0.
    pub index: usize,

    /// The amount of time white has, in milliseconds.
    pub white_time: u32,

    /// The amount of time black has, in milliseconds.
    pub black_time: u32,

    /// Whether the clock is ticking.
    pub clock_is_ticking: bool,
}

impl Cursor {
    pub fn new(state: BoardState) -> Self {
        Self {
            state, 
            ..Self::default()
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            state: BoardState::default(),
            index: 0,
            white_time: 0,
            black_time: 0,
            clock_is_ticking: false,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Branch {
    /// The ID of the parent ChessGame, which may also be a branch.
    pub parent_id: u64,

    /// The halfmove index in the parent that was branched from.
    pub src_index: usize,

    /// The move that took place to reach
    /// the start position in this branch.
    pub delta: BoardDelta,

    /// The halfmove index of the source position in the parent.
    pub src_halfmoves: u16,
}
