use crate::{board::BitBoard, cached::BETWEEN_EXCLUSIVE, square::{File, Rank, Square}, team::Team};


#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CastleRights {
    /// Whether long/short castling is lost for each team.
    pub rights: u8,

    /// The settings that control castling behavior.
    pub settings: CastleSettings,
}

impl CastleRights {
    /// Whether or not this team has castling in this direction.
    pub const fn has(&self, side: Castle, team: Team) -> bool {
        match (side, team) {
            (Castle::Short, Team::White) => self.rights & 0b0001 != 0,
            (Castle::Short, Team::Black) => self.rights & 0b0100 != 0,
            (Castle::Long, Team::White) => self.rights & 0b0010 != 0,
            (Castle::Long, Team::Black) => self.rights & 0b1000 != 0,
        }
    }

    /// Take castle rights for this side and team, returning whether a change occured.
    pub const fn lose(&mut self, side: Castle, team: Team) -> bool {
        if self.has(side, team) {
            match (side, team) {
                (Castle::Short, Team::White) => self.rights ^= 0b0001,
                (Castle::Short, Team::Black) => self.rights ^= 0b0100,
                (Castle::Long, Team::White) => self.rights ^= 0b0010,
                (Castle::Long, Team::Black) => self.rights ^= 0b1000,
            }

            true 
        } else {
            false
        }
    }

    /// Give castle rights for this side and team, returning whether a change occured.
    pub const fn give(&mut self, side: Castle, team: Team) -> bool {
        if !self.has(side, team) {
            match (side, team) {
                (Castle::Short, Team::White) => self.rights ^= 0b0001,
                (Castle::Short, Team::Black) => self.rights ^= 0b0100,
                (Castle::Long, Team::White) => self.rights ^= 0b0010,
                (Castle::Long, Team::Black) => self.rights ^= 0b1000,
            }

            true 
        } else {
            false
        }
    }

    /// Set the rook start file for this side.
    pub fn set_rook(&mut self, side: Castle, file: u8) {
        match side {
            Castle::Long => self.settings.long_file = file.into(),
            Castle::Short => self.settings.short_file = file.into(),
        }
    }

    /// Get the start square of the rook on this side on this team.
    pub fn rook_start(&self, side: Castle, team: Team) -> Square {
        match side {
            Castle::Short => (team.back_rank(), self.settings.short_file),
            Castle::Long => (team.back_rank(), self.settings.long_file),
        }.into()
    }

    /// Get the square the rook would move to if castle occurred.
    pub fn rook_target(&self, side: Castle, team: Team) -> Square {
        match side {
            Castle::Short => (team.back_rank(),File::F),
            Castle::Long => (team.back_rank(), File::D),
        }.into()
    }

    pub fn set_king(&mut self, file: u8) {
        self.settings.king_file = file.into();
    }

    pub fn king_start(&self, team: Team) -> Square {
        Square::new(team.back_rank(), self.settings.king_file)
    }

    /// Get the square the king would move to if castle occurred.
    pub fn king_target(&self, side: Castle, team: Team) -> Square {
        match side {
            Castle::Short => (team.back_rank(), File::G),
            Castle::Long => (team.back_rank(), File::C),
        }.into()
    }

    /// Squares that must not be defended for castling to occur.
    pub fn required_unchecked_squares(&self, king: Square, side: Castle, team: Team) -> BitBoard {
        let tar = self.king_target(side, team);
        BitBoard(BETWEEN_EXCLUSIVE[tar.to_index()][king.to_index()]) | king | tar
    }

    /// Squares that must noe be occupied for castling to occur.
    pub fn required_unoccupied_squares(&self, king: Square, side: Castle, team: Team) -> BitBoard {
        let rook_src = self.rook_start(side, team);
        let rook_tar = self.rook_target(side, team);
        let king_tar = self.king_target(side, team);
        BitBoard(
            BETWEEN_EXCLUSIVE[rook_src.to_index()][rook_tar.to_index()] |
            BETWEEN_EXCLUSIVE[king.to_index()][king_tar.to_index()]
        )
        .without(rook_src)
        .without(king)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Castle {
    /// Kingside castle
    Short,

    /// Queenside castle.
    Long,
}

impl Default for CastleRights {
    fn default() -> Self {
        Self {
            rights: 0b1111,
            settings: CastleSettings::default(),
        }
    }
}

pub fn can_castle(
    side: Castle, 
    team: Team, 
    rights: CastleRights, 
    defense: BitBoard, 
    occupied: BitBoard,
    king: Square,
) -> bool {
    rights.has(side, team) &&
    !defense.intersects(rights.required_unchecked_squares(king, side, team)) && 
    !occupied.intersects(rights.required_unoccupied_squares(king, side, team)) 
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CastleSettings {
    /// The File the king starts on.
    pub king_file: File,

    /// The start file of the kingside rook.
    pub short_file: File,

    /// The start file of the queenside rook.
    pub long_file: File,
}

impl Default for CastleSettings {
    fn default() -> Self {
        Self {
            king_file: File::E,
            short_file: File::H,
            long_file: File::A,
        }
    }
}
