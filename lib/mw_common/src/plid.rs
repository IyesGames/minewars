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
    Neutral,
    Player(NonZeroU8),
}

impl PlayerId {
    pub fn i(self) -> usize {
        self.into()
    }
}

impl From<PlayerId> for u8 {
    fn from(plid: PlayerId) -> u8 {
        match plid {
            PlayerId::Neutral => 0,
            PlayerId::Player(plid) => plid.into(),
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
            None => PlayerId::Neutral,
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

/// Bitmask to mux player IDs: can support a game with up to 7 players
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Plids(u8);

impl Plids {
    const MAX_PLID: u8 = 7;

    pub fn all(with_spect: bool) -> Self {
        Self(!((!with_spect) as u8))
    }

    pub fn spect() -> Self {
        Self::from(PlayerId::Neutral)
    }

    pub fn with_spect(plid: PlayerId) -> Self {
        Self::from(plid) + PlayerId::Neutral
    }

    pub fn contains(&self, plid: PlayerId) -> bool {
        let b: u8 = plid.into();
        (self.0 & (1 << b)) != 0
    }

    pub fn contains_all(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn contains_any(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn iter(&self, max: Option<u8>) -> PlidsIter {
        PlidsIter {
            plids: *self,
            next: 0,
            max: max.unwrap_or(Self::MAX_PLID).min(Self::MAX_PLID),
        }
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

pub struct PlidsIter {
    plids: Plids,
    next: u8,
    max: u8,
}

impl Iterator for PlidsIter {
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

