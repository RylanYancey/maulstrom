use crate::{board::BitBoard, square::Square, team::Team};


#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Piece {
    Bishop,
    Knight,
    Queen,
    King,
    Rook,
    Pawn,
}

impl Piece {
    pub fn to_char_lower(&self) -> char {
        match self {
            Self::Bishop => 'b',
            Self::Knight => 'n',
            Self::Queen => 'q',
            Self::King => 'k',
            Self::Rook => 'r',
            Self::Pawn => 'p'
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Bishop => 0,
            Self::Knight => 1,
            Self::Queen => 2,
            Self::Rook => 3,
            Self::King => 4,
            Self::Pawn => 5,
        }
    }

    pub fn from_u8(u: u8) -> Option<Self> {
        Some(
            match u {
                0 => Self::Bishop,
                1 => Self::Knight,
                2 => Self::Queen,
                3 => Self::King,
                4 => Self::Rook,
                5 => Self::Pawn,
                _ => return None
            }
        )
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Pieces {
    pub bishops: BitBoard,
    pub knights: BitBoard,
    pub queens: BitBoard,
    pub kings: BitBoard,
    pub rooks: BitBoard,
    pub pawns: BitBoard,
    pub white: BitBoard,
    pub black: BitBoard,
}   

impl Pieces {
    pub fn just_pawns() -> Self {
        Self {
            bishops: BitBoard::new(),
            knights: BitBoard::new(),
            queens: BitBoard::new(),
            kings: BitBoard::new(),
            rooks: BitBoard::new(),
            pawns: BitBoard::new().with_rank(1).with_rank(6),
            white: BitBoard::new().with_rank(1),
            black: BitBoard::new().with_rank(6),
        }
    }

    pub fn get(&self, piece: Piece, team: Team) -> BitBoard {
        self.index(piece) & self.on_team(team)
    }
    
    pub fn all(&self, pieces: &[Piece], team: Team) -> BitBoard {
        pieces.iter().fold(BitBoard::new(), |all, pc| all | self.index(*pc)) & self.on_team(team)
    }

    pub fn occupied(&self) -> BitBoard {
        self.white | self.black
    }

    pub fn on_team(&self, team: Team) -> BitBoard {
        match team {
            Team::White => self.white,
            Team::Black => self.black,
        }
    }

    pub fn piece_at(&self, at: Square) -> Option<Piece> {
        if !(self.white | self.black).has(at) { return None }
        if self.bishops.has(at) { return Some(Piece::Bishop) }
        if self.knights.has(at) { return Some(Piece::Knight) }
        if self.queens.has(at) { return Some(Piece::Queen) }
        if self.kings.has(at) { return Some(Piece::King) }
        if self.rooks.has(at) { return Some(Piece::Rook) }
        if self.pawns.has(at) { return Some(Piece::Pawn) }
        None
    }

    /// If holes has at, then every hole is searched for a piece.
    pub fn piece_at_or_on_hole(&self, at: Square, holes: BitBoard) -> Option<Piece> {
        if holes.has(at) {
            if !(self.white | self.black).intersects(holes) { return None }
            if self.bishops.intersects(holes) { return Some(Piece::Bishop) }
            if self.knights.intersects(holes) { return Some(Piece::Knight) }
            if self.queens.intersects(holes) { return Some(Piece::Queen) }
            if self.kings.intersects(holes) { return Some(Piece::King) }
            if self.rooks.intersects(holes) { return Some(Piece::Rook) }
            if self.pawns.intersects(holes) { return Some(Piece::Pawn) }
            None
        } else {
            self.piece_at(at)
        }
    }

    /// Remove the piece at the square. If the piece is on a wormhole,
    /// all wormhole squares are checked.
    pub fn remove(&mut self, at: Square, holes: BitBoard) -> Option<Piece> {
        let sqs = if holes.has(at) { holes } else { BitBoard::from(at) };
        if self.white.intersects(sqs) { self.white.clear_all(sqs); }
        if self.black.intersects(sqs) { self.black.clear_all(sqs); }
        if self.bishops.intersects(sqs) { return self.bishops.clear_all(sqs).then_some(Piece::Bishop); }
        if self.knights.intersects(sqs) { return self.knights.clear_all(sqs).then_some(Piece::Knight) }
        if self.queens.intersects(sqs) { return self.queens.clear_all(sqs).then_some(Piece::Queen) }
        if self.kings.intersects(sqs) { return self.kings.clear_all(sqs).then_some(Piece::King) }
        if self.rooks.intersects(sqs) { return self.rooks.clear_all(sqs).then_some(Piece::Rook) }
        if self.pawns.intersects(sqs) { return self.pawns.clear_all(sqs).then_some(Piece::Pawn) }
        None
    }

    /// Insert a piece at the square, clearing any pieces already on the square.
    pub fn insert(&mut self, at: Square, pc: Piece, team: Team, holes: BitBoard) {
        let i = if holes.has(at) { holes | at } else { at.into() };
        if (self.white | self.black).intersects(i) {
            let j = !i;
            self.white &= j;
            self.black &= j;
            self.bishops &= j;
            self.knights &= j;
            self.queens &= j;
            self.kings &= j;
            self.rooks &= j;
            self.pawns &= j;
        }

        match team {
            Team::White => self.white |= at,
            Team::Black => self.black |= at
        }
        match pc {
            Piece::Bishop => self.bishops |= at,
            Piece::Knight => self.knights |= at,
            Piece::Queen => self.queens |= at,
            Piece::King => self.kings |= at,
            Piece::Rook => self.rooks |= at,
            Piece::Pawn => self.pawns |= at,
        }
    }

    fn index(&self, piece: Piece) -> BitBoard {
        match piece {
            Piece::Bishop => self.bishops,
            Piece::Knight => self.knights,
            Piece::Queen => self.queens,
            Piece::Rook => self.rooks,
            Piece::King => self.kings,
            Piece::Pawn => self.pawns,
        }
    }

    pub fn setup_from_file(&mut self, piece: Piece, file: u8) {
        let sqs = BitBoard::new() | (0, file as i8) | (7, file as i8);
        self.white |= (0, file as i8);
        self.black |= (7, file as i8);
        *match piece {
            Piece::Bishop => &mut self.bishops,
            Piece::Knight => &mut self.knights,
            Piece::Queen => &mut self.queens,
            Piece::King => &mut self.kings,
            Piece::Rook => &mut self.rooks,
            Piece::Pawn => &mut self.pawns,
        } |= sqs;
    }
}

impl Default for Pieces {
    fn default() -> Self {
        Self {
            knights: BitBoard(0x4200000000000042),
            bishops: BitBoard(0x2400000000000024),
            queens: BitBoard(0x800000000000008),
            kings: BitBoard(0x1000000000000010),
            pawns: BitBoard(0x00FF00000000FF00),
            rooks: BitBoard(0x8100000000000081),
            white: BitBoard(0x000000000000FFFF),
            black: BitBoard(0xFFFF000000000000),
        }
    }
}