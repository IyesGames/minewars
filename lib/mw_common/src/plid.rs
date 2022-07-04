//! Player IDs
//!
//! This module contains the types used for tracking all the different views
//! of a multiplayer game. [`PlayerId`] represents one "view" of the game:
//! either the global spectator view, or the view of a specific player.
//! [`Plids`] and [`PlidsBig`] represent a set of views. Useful, for example,
//! when a message from the server should be sent to multiple players.

use std::num::NonZeroU8;
use std::ops::{Add, AddAssign, Sub, SubAssign, Not};

/// Player ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlayerId {
    Spectator,
    Player(NonZeroU8),
}

impl PlayerId {
    pub fn i(self) -> usize {
        self.into()
    }
}

impl From<PlayerId> for u8 {
    fn from(plid: PlayerId) -> u8 {
        use PlayerId::*;
        match plid {
            Spectator => 0,
            Player(plid) => plid.into(),
        }
    }
}

impl From<PlayerId> for usize {
    fn from(plid: PlayerId) -> usize {
        u8::from(plid) as usize
    }
}

impl From<u8> for PlayerId {
    fn from(b: u8) -> PlayerId {
        match NonZeroU8::new(b) {
            Some(plid) => PlayerId::Player(plid),
            None => PlayerId::Spectator,
        }
    }
}

impl PartialEq<u8> for PlayerId {
    fn eq(&self, other: &u8) -> bool {
        u8::from(*self) == *other
    }
}

impl PartialEq<PlayerId> for u8 {
    fn eq(&self, other: &PlayerId) -> bool {
        u8::from(*other) == *self
    }
}

pub trait PlidMask:
    Send + Sync + Copy + Eq + Default
    + Add<PlayerId> + Sub<PlayerId>
    + AddAssign<PlayerId> + SubAssign<PlayerId>
    + From<PlayerId> + Not
{
    const MAX_PLID: u8;

    /// convenience helper: Broadcast to everyone
    fn all(with_spect: bool) -> Self;
    /// convenience helper: Spectator-only
    fn spect() -> Self;
    /// convenience helper: Player + Spectators
    fn with_spect(plid: PlayerId) -> Self;
    fn contains(&self, plid: PlayerId) -> bool;
    fn contains_any(&self, other: Self) -> bool;
    fn contains_all(&self, other: Self) -> bool;

    fn iter(&self, max: Option<u8>) -> PlidsIter<Self> {
        PlidsIter {
            plids: *self,
            next: 0,
            max: max.unwrap_or(Self::MAX_PLID).min(Self::MAX_PLID),
        }
    }
}

/// Regular mask: can support a game with up to 7 players
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Plids(u8);
/// Big mask: can support huge game modes like battle-royale with up to 127 players
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlidsBig(u128);

impl PlidMask for PlidsBig {
    const MAX_PLID: u8 = 127;

    fn all(with_spect: bool) -> Self {
        Self(!((!with_spect) as u128))
    }

    fn spect() -> Self {
        Self::from(PlayerId::Spectator)
    }

    fn with_spect(plid: PlayerId) -> Self {
        Self::from(plid) + PlayerId::Spectator
    }

    fn contains(&self, plid: PlayerId) -> bool {
        let b: u8 = plid.into();
        let b: u128 = b.into();
        (self.0 & (1 << b)) != 0
    }

    fn contains_all(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    fn contains_any(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl PlidMask for Plids {
    const MAX_PLID: u8 = 7;

    fn all(with_spect: bool) -> Self {
        Self(!((!with_spect) as u8))
    }

    fn spect() -> Self {
        Self::from(PlayerId::Spectator)
    }

    fn with_spect(plid: PlayerId) -> Self {
        Self::from(plid) + PlayerId::Spectator
    }

    fn contains(&self, plid: PlayerId) -> bool {
        let b: u8 = plid.into();
        (self.0 & (1 << b)) != 0
    }

    fn contains_all(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    fn contains_any(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl From<PlayerId> for Plids {
    fn from(plid: PlayerId) -> Self {
        Self::default() + plid
    }
}

impl Add<PlayerId> for Plids {
    type Output = Self;

    fn add(self, rhs: PlayerId) -> Self {
        let b: u8 = rhs.into();
        Self(self.0 | (1 << b))
    }
}

impl AddAssign<PlayerId> for Plids {
    fn add_assign(&mut self, rhs: PlayerId) {
        *self = *self + rhs;
    }
}

impl Add<Plids> for Plids {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl AddAssign<Plids> for Plids {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub<PlayerId> for Plids {
    type Output = Self;

    fn sub(self, rhs: PlayerId) -> Self {
        let b: u8 = rhs.into();
        Self(self.0 & !(1 << b))
    }
}

impl SubAssign<PlayerId> for Plids {
    fn sub_assign(&mut self, rhs: PlayerId) {
        *self = *self - rhs;
    }
}

impl Sub<Plids> for Plids {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
}

impl SubAssign<Plids> for Plids {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Not for Plids {
    type Output = Self;

    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl From<PlayerId> for PlidsBig {
    fn from(plid: PlayerId) -> Self {
        Self::default() + plid
    }
}

impl Add<PlayerId> for PlidsBig {
    type Output = Self;

    fn add(self, rhs: PlayerId) -> Self {
        let b: u8 = rhs.into();
        let b: u128 = b.into();
        Self(self.0 | (1 << b))
    }
}

impl AddAssign<PlayerId> for PlidsBig {
    fn add_assign(&mut self, rhs: PlayerId) {
        *self = *self + rhs;
    }
}

impl Add<PlidsBig> for PlidsBig {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl AddAssign<PlidsBig> for PlidsBig {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub<PlayerId> for PlidsBig {
    type Output = Self;

    fn sub(self, rhs: PlayerId) -> Self {
        let b: u8 = rhs.into();
        let b: u128 = b.into();
        Self(self.0 & !(1 << b))
    }
}

impl SubAssign<PlayerId> for PlidsBig {
    fn sub_assign(&mut self, rhs: PlayerId) {
        *self = *self - rhs;
    }
}

impl Sub<PlidsBig> for PlidsBig {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
}

impl SubAssign<PlidsBig> for PlidsBig {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Not for PlidsBig {
    type Output = Self;

    fn not(self) -> Self {
        Self(!self.0)
    }
}

pub struct PlidsIter<P: PlidMask> {
    plids: P,
    next: u8,
    max: u8,
}

impl<P: PlidMask> Iterator for PlidsIter<P> {
    type Item = PlayerId;

    fn next(&mut self) -> Option<PlayerId> {
        let ret = loop {
            if self.next > self.max {
                return None;
            }

            let cur = self.next;
            self.next += 1;

            if self.plids.contains(cur.into()) {
                break cur;
            }
        };

        Some(ret.into())
    }
}
