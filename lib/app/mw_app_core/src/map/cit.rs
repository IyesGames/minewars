//! All the various things we keep track of for Cits on the map
//!
//! Every Cit has its own entity to represent it.
//!
//! To find the entity for a specific Cit, look it up via the
//! `CitIndex` on the Map Governor.

use mw_common::plid::PlayerId;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    // TODO: maybe rc?
    app.configure_stage_set_no_rc(Update, CitUpdateSS);
}

/// Anything that updates components on cit entities
#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct CitUpdateSS;

#[derive(Bundle)]
pub struct CitBundle {
    pub cleanup: GamePartialCleanup,
    pub marker: MwCit,
    pub region: CitRegion,
    pub owner: CitOwner,
    pub economy: CitEconomy,
}

#[derive(Component)]
pub struct MwCit;

#[derive(Component)]
pub struct CitRegion(pub u8);

#[derive(Component)]
pub struct CitOwner(pub PlayerId);

#[derive(Component)]
pub struct CitEconomy {
    pub money: u32,
    pub income: u16,
    pub res: u16,
    pub import: u8,
    pub export: u8,
}
