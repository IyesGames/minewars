use mw_app_core::{driver::GameOutEventSS, session::{PlayersIndex, PlidViewing, SessionGovernor}, view::*};
use mw_common::game::{event::{GameEvent, MwEv}, ItemKind};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        view_update_from_gameevents,
    )
        .in_set(MultiViewEnabledSet)
        .in_set(SetStage::Provide(ViewSS::Update))
        .in_set(SetStage::WantChanged(GameOutEventSS))
    );
    app.add_systems(Update, (
        switch_view_despawn,
        switch_view_showhide,
        // TODO: map update systems
    )
        .in_set(MultiViewEnabledSet)
        .in_set(SetStage::Provide(ViewSS::Switch))
        .run_if(rc_viewswitch)
    );
}

fn rc_viewswitch(
    q_session: Query<Ref<PlidViewing>, With<SessionGovernor>>,
) -> bool {
    q_session.get_single()
        .map(|plid| plid.is_changed())
        .unwrap_or(false)
}

fn switch_view_despawn(
    mut commands: Commands,
    q: Query<Entity, With<DespawnOnViewSwitch>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn switch_view_showhide(
    q_session: Query<&PlidViewing, With<SessionGovernor>>,
    mut q: Query<(&mut Visibility, &VisibleInView)>,
) {
    let viewing = q_session.single();
    for (mut vis, viewvis) in &mut q {
        if viewvis.0 == viewing.0 {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn view_update_from_gameevents(
    mut evr: EventReader<GameEvent>,
    q_session: Query<&PlayersIndex, With<SessionGovernor>>,
    mut q_view: Query<&mut ViewMapData>,
) {
    let index = q_session.single();
    for ev in evr.read() {
        let plid = ev.plid;
        // Ignore event if we don't have a view for that plid set up
        let Some(e_plid) = index.e_plid.get(plid.i()) else {
            continue;
        };
        let Ok(mut view) = q_view.get_mut(*e_plid) else {
            continue;
        };
        match ev.ev {
            MwEv::TileKind { pos, kind } => {
                let tile = &mut view.0[pos];
                tile.set_kind(kind);
            },
            MwEv::TileOwner { pos, plid } => {
                let tile = &mut view.0[pos];
                tile.set_owner(u8::from(plid));
            },
            MwEv::DigitCapture { pos, digit, asterisk } => {
                let tile = &mut view.0[pos];
                tile.set_owner(u8::from(plid));
                tile.set_digit(digit);
                tile.set_asterisk(asterisk);
            },
            MwEv::RevealItem { pos, item } => {
                let tile = &mut view.0[pos];
                tile.set_item(item);
            },
            MwEv::Flag { pos, plid } => {
                let tile = &mut view.0[pos];
                tile.set_flag(u8::from(plid));
            },
            MwEv::Explode { pos } => {
                let tile = &mut view.0[pos];
                // clear any item from the tile
                tile.set_item(ItemKind::Safe);
            },
            _ => {}
        }
    }
}
