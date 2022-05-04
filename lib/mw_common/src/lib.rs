pub mod plid;
pub mod grid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineKind {
    Mine,
    Decoy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoadState {
    Pending,
    Built,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Water,
    Land,
    Fertile,
    Mountain,
    Road,
}
