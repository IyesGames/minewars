use mw_common::plid::PlayerId;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
}

#[derive(Bundle)]
pub struct CitBundle {
    pub region: CitRegion,
    pub owner: CitOwner,
    pub economy: CitEconomy,
}

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
