use chrono::Duration;

use crate::{board::BitBoard, castle::{Castle, CastleRights}, end::EndCondition, pieces::Piece, settings::GameSettings, square::Square, state::BoardState, trace::MoveTrace};

#[derive(Clone)]
pub struct ChessGame {
    /// The state of the board on the first move.
    pub start: BoardState,

    /// The position currently being viewed.
    pub cursor: Cursor,

    /// The changes that occur at each move in the game.
    /// If this game is a Branch, the first element will
    /// be the delta taken to reach the start position. If this is not
    /// a branch the length will not contain a delta for the
    /// starting position.
    pub deltas: Vec<BoardDelta>,

    /// Configuration of game settings.
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
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_is_last(&self) -> bool {
        if self.is_branch.is_some() {
            self.cursor.index == self.deltas.len() - 1
        } else {
            self.cursor.index == self.deltas.len()
        }
    }

    /// Get the delta for the move that was played in this position.
    /// If this is the last move in the game, None is returned.
    pub fn get_next_delta(&self) -> Option<BoardDelta> {
        (!self.cursor_is_last()).then(|| {
            if self.is_branch.is_some() {
                self.deltas[self.cursor.index - 1]
            } else {
                self.deltas[self.cursor.index]
            }
        })
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
            if trace.requires_promotion && promote.is_none_or(|pc| pc == Piece::Pawn) {
                return Err(PlayError::RequiresPromotion)
            }

            let mut next = self.cursor.state;
            let mut delta = BoardDelta(0);
            let mut castle = next.castle;
            let mut halfmoves = next.halfmoves + 1;

            delta.set_src_sq(src);
            delta.set_dst_sq(dst);

            if trace.is_king_move {
                castle.lose(Castle::Long, next.turn);
                castle.lose(Castle::Short, next.turn);
    
                if let Some(side) = trace.is_castle {
                    castle.lose(side, next.turn);
                    delta.set_src_sq(castle.king_start(next.turn));
                    delta.set_dst_sq(castle.rook_target(side, next.turn));
                    delta.set_flags(DeltaFlags::IS_CASTLE);
                } else {
                    if let Some(side) = trace.loses_castle {
                        castle.lose(side, next.turn);
                    }
        
                    if let Some(side) = trace.takes_castle {
                        castle.lose(side, !next.turn);
                    }
                }

                if castle.rights != next.castle.rights {
                    halfmoves = 0;
                }
            } else {
                if next.pieces.pawns.has(src) || trace.captures.is_some() {
                    halfmoves = 0;
                }
            }

            delta.set_castle_rights(castle.rights);
            delta.set_capture_pc(trace.captures);
            delta.set_prev_ep_sq(next.en_passant);
            delta.set_prev_halfmoves(next.halfmoves);

            if trace.requires_promotion {
                delta.set_promote_pc(promote);
                delta.add_flags(DeltaFlags::IS_PROMOTE);
            }

            todo!()
        } else {
            Err(PlayError::InvalidMove)
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

    /// The halfmove index of the cursor position.
    pub index: usize,

    /// The amount of time white has, in milliseconds.
    pub white_time: i64,

    /// The amount of time black has, in milliseconds.
    pub black_time: i64,

    /// Whether the clock is ticking.
    pub clock_is_ticking: bool,
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DeltaFlags(pub u16);

bitflags::bitflags! {
    impl DeltaFlags: u16 {
        /// Whether the move is a capture en-passant.
        const IS_EN_PASSANT = 1;

        /// Whether the move is a double-push of a pawn,
        /// which allows en-passant.
        const IS_DOUBLE_PUSH = 1<<1;

        /// If the position the move was played in had en passant,
        /// then it is stored in this delta. 
        const HAS_PREV_EP_SQ = 1<<2;

        /// The move took longer than 163840 milliseconds, so the stored
        /// time is in seconds rather than millis.
        const TIME_SECONDS = 1<<3;

        /// The move causes the halfmoves, which are used to check for
        /// the fifty-move-rule, to reset.
        const RESET_HALFMOVES = 1<<4;

        /// Indicates the move is a capture. This also means
        /// the piece type the pawn promoted to is stored in this delta,
        /// accessible via BoardDelta::get_capture_pc.
        const IS_CAPTURE = 1<<5;

        /// Indicates the move is a pawn promotion. This also means
        /// the piece type the pawn promoted to is stored in this delta.
        /// Accessible via BoardDelta::get_promote_pc.
        const IS_PROMOTE = 1<<6;

        /// Indicates the move is castle.
        const IS_CASTLE = 1<<7;

        /// Indicates a hole was "pushed" onto the queue. This means 
        /// the hole should be visible to the players, but not yet made 
        /// available for usage.
        const HOLE_PUSHED = 1<<8;

        /// Indicates a hole was "popped" off the queue. This means the hole,
        /// which was previously queued, should now be available for usage by
        /// both parties.
        const HOLE_POPPED = 1<<9;

        /// Indicates a hole will be popped off the queue next turn.
        /// This is used to determine what moves are defended next turn, so
        /// the king is forced to move off of squares that will be defended.
        const HOLE_WILL_POP = 1<<10;
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BoardDelta(u64);

impl BoardDelta {
    const FLAG_LEN: usize = 11;
    const FLAG_OFFS: usize = 0;

    const SRC_SQ_LEN: usize = 6;
    const SRC_SQ_OFFS: usize = Self::FLAG_LEN;

    const DST_SQ_LEN: usize = 6;
    const DST_SQ_OFFS: usize = Self::SRC_SQ_OFFS + Self::SRC_SQ_LEN;

    const TIME_LEN: usize = 13;
    const TIME_OFFS: usize = Self::DST_SQ_OFFS + Self::DST_SQ_LEN;

    const CAPTURE_LEN: usize = 3;
    const CAPTURE_OFFS: usize = Self::TIME_OFFS + Self::TIME_LEN;

    /// Can't promote to a King or Pawn, only 4 possibilities.
    const PROMOTE_LEN: usize = 2;
    const PROMOTE_OFFS: usize = Self::CAPTURE_OFFS + Self::CAPTURE_LEN;

    const PREV_EP_SQ_LEN: usize = 6;
    const PREV_EP_SQ_OFFS: usize = Self::PROMOTE_OFFS + Self::PROMOTE_LEN;

    const CASTLE_LEN: usize = 4;
    const CASTLE_OFFS: usize = Self::PREV_EP_SQ_OFFS + Self::PREV_EP_SQ_LEN;

    const HOLE_SQ_LEN: usize = 6;
    const HOLE_SQ_OFFS: usize = Self::CASTLE_OFFS + Self::CASTLE_LEN;

    const HALFMOVES_LEN: usize = 6;
    const HALFMOVES_OFFS: usize = Self::HOLE_SQ_OFFS + Self::HOLE_SQ_LEN;

    #[inline(always)]
    const fn extract(val: u64, len: usize, offs: usize) -> u64 {
        (val >> offs) & ((1 << len) - 1)
    }

    #[inline(always)]
    const fn deposit(val: u64, len: usize, offs: usize, item: u64) -> u64 {
        (val & !(((1 << len) - 1) << offs)) | item << offs
    }

    pub fn get_flags(&self) -> DeltaFlags {
        DeltaFlags(Self::extract(self.0, Self::FLAG_LEN, Self::FLAG_OFFS) as u16)
    }

    pub fn set_flags(&mut self, flags: DeltaFlags) {
        self.0 = Self::deposit(self.0, Self::FLAG_LEN, Self::FLAG_OFFS, flags.0 as u64)
    }

    pub fn add_flags(&mut self, flags: DeltaFlags) {
        self.0 = Self::deposit(self.0, Self::FLAG_LEN, Self::FLAG_OFFS, (flags | self.get_flags()).0 as u64)
    }

    pub fn get_src_sq(&self) -> Square {
        Square::from_index(Self::extract(self.0, Self::SRC_SQ_LEN, Self::SRC_SQ_OFFS) as usize)
    }

    /// Set the source square.
    /// If the move is castling, this should be the square of the king.
    pub fn set_src_sq(&mut self, src: Square) {
        self.0 = Self::deposit(self.0, Self::SRC_SQ_LEN, Self::SRC_SQ_OFFS, src.to_index() as u64);
    }

    /// Get the time, in milliseconds, the move took.
    pub fn get_time(&self) -> u64 {
        let t = Self::extract(self.0, Self::TIME_LEN, Self::TIME_OFFS);
        if self.get_flags().contains(DeltaFlags::TIME_SECONDS) {
            t * 1000
        } else {
            t * 10
        }
    }

    /// Set the time, in milliseconds, the move took.
    pub fn set_time(&mut self, mut time: u64) {
        let mut flags = self.get_flags();
        if time >= (1 << Self::TIME_LEN as u64) * 10 {
            self.set_flags(flags | DeltaFlags::TIME_SECONDS);
            time = time / 1000;
        } else {
            flags.remove(DeltaFlags::TIME_SECONDS);
            self.set_flags(flags);
            // in actuality, the stored time is in 1/100th of millisecond steps.
            time = time / 10;
        }
        self.0 = Self::deposit(self.0, Self::TIME_LEN, Self::TIME_OFFS, time)
    }

    pub fn get_dst_sq(&self) -> Square {
        Square::from_index(Self::extract(self.0, Self::DST_SQ_LEN, Self::DST_SQ_OFFS) as usize)
    }

    /// Set the destination square.
    /// If the move is castling, this should be the square of the rook we are castling with.
    pub fn set_dst_sq(&mut self, dst: Square) {
        self.0 = Self::deposit(self.0, Self::DST_SQ_LEN, Self::DST_SQ_OFFS, dst.to_index() as u64);
    }

    /// Get the piece that was captured.
    pub fn get_capture_pc(&self) -> Option<Piece> {
        self.get_flags().contains(DeltaFlags::IS_CAPTURE).then(|| {
            Piece::from_u8(Self::extract(self.0, Self::CAPTURE_LEN, Self::CAPTURE_OFFS) as u8)
        })
    }

    /// Set the piece that was captured.
    pub fn set_capture_pc(&mut self, pc: Option<Piece>) {
        if pc.is_some() {
            self.set_flags(self.get_flags() | DeltaFlags::IS_CAPTURE);
            self.0 = Self::deposit(self.0, Self::CAPTURE_LEN, Self::CAPTURE_OFFS, pc.map(|pc| pc.to_u8() as u64).unwrap_or(0));
        } else {
            let mut flags = self.get_flags();
            flags.remove(DeltaFlags::IS_CAPTURE);
            self.set_flags(flags);
        }
    }

    /// Get the piece that the pawn promoted to, if relevant.
    pub fn get_promote_pc(&self) -> Option<Piece> {
        self.get_flags().contains(DeltaFlags::IS_PROMOTE).then(|| {
            Piece::from_u8(Self::extract(self.0, Self::PROMOTE_LEN, Self::PROMOTE_OFFS) as u8)
        })
    }

    pub fn set_promote_pc(&mut self, pc: Option<Piece>) {
        if pc.is_some() {
            self.set_flags(self.get_flags() | DeltaFlags::IS_PROMOTE);
            self.0 = Self::deposit(self.0, Self::PROMOTE_LEN, Self::PROMOTE_OFFS, pc.map(|pc| pc.to_u8() as u64).unwrap_or(0));
        } else {
            let mut flags = self.get_flags();
            flags.remove(DeltaFlags::IS_PROMOTE);
            self.set_flags(flags);
        }
    }

    /// Get the en-passant square in the position the move was played.
    pub fn get_prev_ep_sq(&self) -> Option<Square> {
        self.get_flags().contains(DeltaFlags::IS_DOUBLE_PUSH).then(|| {
            Square::from_index(Self::extract(self.0, Self::PREV_EP_SQ_LEN, Self::PREV_EP_SQ_OFFS) as usize)
        })
    }

    pub fn set_prev_ep_sq(&mut self, sq: Option<Square>) {
        if sq.is_some() {
            self.set_flags(self.get_flags() | DeltaFlags::IS_DOUBLE_PUSH);
            self.0 = Self::deposit(self.0, Self::PREV_EP_SQ_LEN, Self::PREV_EP_SQ_OFFS, sq.map(|pc| pc.to_index() as u64).unwrap_or(0));
        } else {
            let mut flags = self.get_flags();
            flags.remove(DeltaFlags::IS_DOUBLE_PUSH);
            self.set_flags(flags);
        }
    }

    /// Get the castling rights in the RESULTING position.
    pub fn get_castle_rights(&self) -> u8 {
        Self::extract(self.0, Self::CASTLE_LEN, Self::CASTLE_OFFS) as u8
    }

    pub fn set_castle_rights(&mut self, rights: u8) {
        self.0 = Self::deposit(self.0, Self::CASTLE_LEN, Self::CASTLE_OFFS, rights as u64)
    }

    /// Get the square of a queued wormhole placement.
    pub fn get_hole_square(&self) -> Option<Square> {
        let flags = self.get_flags();
        (flags.contains(DeltaFlags::HOLE_PUSHED) || flags.contains(DeltaFlags::HOLE_POPPED)) .then(|| {
            Square::from_index(Self::extract(self.0, Self::HOLE_SQ_LEN, Self::HOLE_SQ_OFFS) as usize)
        })
    }

    /// Set the square of a queued wormhole placement.
    pub fn set_pushed_hole(&mut self, sq: Option<Square>) {
        let mut flags = self.get_flags();
        if sq.is_some() {
            self.set_flags(flags | DeltaFlags::HOLE_PUSHED);
            self.0 = Self::deposit(
                self.0, Self::HOLE_SQ_LEN, Self::HOLE_SQ_OFFS, 
                sq.map(|s| s.to_index() as u64).unwrap_or(0)
            );
        } else {
            flags.remove(DeltaFlags::HOLE_PUSHED);
            flags.remove(DeltaFlags::HOLE_POPPED);
            self.set_flags(flags);
        }
    }

    /// Set the square of the wormhole that was just popped.
    pub fn set_popped_hole(&mut self, sq: Option<Square>) {
        let mut flags = self.get_flags();
        if sq.is_some() {
            self.set_flags(flags | DeltaFlags::HOLE_POPPED);
            self.0 = Self::deposit(
                self.0, Self::HOLE_SQ_LEN, Self::HOLE_SQ_OFFS, 
                sq.map(|s| s.to_index() as u64).unwrap_or(0)
            );
        } else {
            flags.remove(DeltaFlags::HOLE_PUSHED);
            flags.remove(DeltaFlags::HOLE_POPPED);
            self.set_flags(flags);
        }
    }

    pub fn get_prev_halfmoves(&self) -> u8 {
        Self::extract(self.0, Self::HALFMOVES_LEN, Self::HALFMOVES_OFFS) as u8
    }

    pub fn set_prev_halfmoves(&mut self, halfmoves: u8) {
        self.0 = Self::deposit(self.0, Self::HALFMOVES_LEN, Self::HALFMOVES_OFFS, halfmoves as u64);
    }

    pub fn castle_side(&self) -> Option<Castle> {
        self.get_flags().contains(DeltaFlags::IS_CASTLE).then(|| {
            if self.get_src_sq() > self.get_dst_sq() {
                Castle::Long
            } else {
                Castle::Short
            }
        })
    }
}

impl Default for BoardDelta {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Branch {
    /// The ID of the parent ChessGame, which may also be a branch.
    pub parent_id: u64,

    /// The halfmove index in the parent that was branched from.
    pub src_index: usize,
}
