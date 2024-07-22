use mw_app_core::map::cit::CitOwner;
use mw_app_core::map::tile::MwMapTile;
use mw_app_core::map::tile::TileOwner;
use mw_app_core::player::*;
use mw_app_core::session::*;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        plid_score_by_cits
            .in_set(InStateSet(AppState::InGame))
            .run_if(any_filter::<(With<PlidScoreByCits>, With<SessionGovernor>)>),
        plid_score_by_owned_pct
            .in_set(InStateSet(AppState::InGame))
            .run_if(any_filter::<(With<PlidScoreByOwnedPct>, With<SessionGovernor>)>),
    ));
}

fn plid_score_by_cits(
    mut q_plid: Query<&mut PlidScore, With<Plid>>,
    q_cit: Query<&CitOwner>,
    q_session: Query<&PlayersIndex, With<SessionGovernor>>,
) {
    // FIXME: PERF: this really shouldnt have to run every frame
    let players = q_session.single();
    q_plid.iter_mut().for_each(|mut score| {
        score.0 = 0;
    });
    for owner in &q_cit {
        let e_plid = players.e_plid[owner.0.i()];
        if let Ok(mut score) = q_plid.get_mut(e_plid) {
            score.0 += 1;
        }
    }
}

fn plid_score_by_owned_pct(
    mut q_plid: Query<&mut PlidScore, With<Plid>>,
    q_tile: Query<&TileOwner, With<MwMapTile>>,
    q_session: Query<&PlayersIndex, With<SessionGovernor>>,
) {
    // FIXME: PERF: this really shouldnt have to run every frame
    let players = q_session.single();
    q_plid.iter_mut().for_each(|mut score| {
        score.0 = 0;
    });
    let mut total = 0;
    for owner in &q_tile {
        let e_plid = players.e_plid[owner.0.i()];
        if let Ok(mut score) = q_plid.get_mut(e_plid) {
            score.0 += 1;
        }
        total += 1;
    }
    q_plid.iter_mut().for_each(|mut score| {
        score.0 = (score.0 as f32 / total as f32 * 100.0) as u32;
    });
}
