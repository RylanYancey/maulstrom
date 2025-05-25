
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EndCondition {
    Checkmate,
    FiftyMoveRule,
    Stalemate,
    Repetition,
    Agreement,
    WhiteResign,
    BlackResign,
}