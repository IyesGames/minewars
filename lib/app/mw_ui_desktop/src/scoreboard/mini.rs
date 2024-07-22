use mw_app_core::{assets::SpritesAssets, player::{Plid, PlidStats}, session::NeedsSessionGovernorSet};
use mw_ui_common::{root::spawn_root, widgets::WidgetsUiUpdateSS};

use crate::{assets::UiAssets, prelude::*, settings::DesktopUiSettings};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::InGame), spawn_miniboard);
    app.add_systems(Update, (
        manage_miniboard_entries
            .before(super::update_plid_icons)
            .in_set(SetStage::Prepare(WidgetsUiUpdateSS)),
        update_miniboard_entries,
        sort_miniboard_entries,
    )
        .chain()
        .run_if(any_with_component::<Miniboard>)
        .in_set(NeedsSessionGovernorSet)
    );
}

#[derive(Component)]
struct PlidEntry(PlayerId, Entity);

#[derive(Component)]
struct Miniboard;

fn spawn_miniboard(
    mut commands: Commands,
) {
    let e_root = spawn_root(&mut commands, Style {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        ..Default::default()
    });
    commands.entity(e_root).insert(GameFullCleanup);
    let e_miniboard = commands.spawn((
        Miniboard,
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    commands.entity(e_root).add_child(e_miniboard);
}

fn manage_miniboard_entries(
    mut commands: Commands,
    settings: Settings,
    assets_spr: Res<SpritesAssets>,
    assets_ui: Res<UiAssets>,
    q_player: Query<(Entity, &Plid), (Added<Plid>, With<PlidStats>)>,
    mut removed: RemovedComponents<Plid>,
    q_entries: Query<(Entity, &PlidEntry)>,
    q_miniboard: Query<Entity, With<Miniboard>>,
) {
    let s_ui = settings.get::<DesktopUiSettings>().unwrap();
    removed.read().for_each(|e_plid| {
        q_entries.iter()
            .filter(|(_, pe)| pe.1 == e_plid)
            .for_each(|(e, _)| commands.entity(e).despawn_recursive());
    });
    q_player.iter().for_each(|(e_plid, plid)| {
        q_miniboard.iter().for_each(|e_mb| {
            let e_entry = spawn_plid_entry(
                &mut commands,
                &assets_spr,
                &assets_ui,
                e_plid, plid.0,
                s_ui.mini_scoreboard_settings.icon_size,
            );
            commands.entity(e_mb).add_child(e_entry);
        });
    });
}

fn update_miniboard_entries(
) {
}

fn sort_miniboard_entries(
) {
}

fn spawn_plid_entry(
    commands: &mut Commands,
    assets_spr: &SpritesAssets,
    assets_ui: &UiAssets,
    e_plid: Entity,
    plid: PlayerId,
    icon_size: f32,
) -> Entity {
    let e_entry = commands.spawn((
        PlidEntry(plid, e_plid),
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    let e_icon_wrap = commands.spawn((
        ImageBundle {
            image: assets_ui.bg_img_miniboard_icon.clone().into(),
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                aspect_ratio: Some(1.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        assets_ui.bg_9p_miniboard_icon.clone(),
    )).id();
    let e_icon = super::spawn_plid_icon(commands, assets_spr, e_plid, plid, icon_size);
    commands.entity(e_icon_wrap).add_child(e_icon);
    commands.entity(e_entry).add_child(e_icon_wrap);
    e_entry
}
