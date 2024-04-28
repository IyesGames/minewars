use mw_common::plid::PlayerId;

use crate::{assets::{GameAssets, UiAssets}, locale::L10nKey, player::{PlayerDisplayName, PlayersIndex}, prelude::*};

pub(super) struct NotifyPlugin;

impl Plugin for NotifyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NotifyEvent>();
        app.configure_stage_set(Update, NotifyEventSS, on_event::<NotifyEvent>());
        app.add_systems(Update, (
            notification_timeout,
            spawn_notifications
                .in_set(SetStage::WantChanged(NotifyEventSS)),
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct NotifyEventSS;

#[derive(Event)]
pub enum NotifyEvent {
    Simple {
        l10n_heading: Option<String>,
        l10n_content: String,
    },
    KillFeed {
        killed: PlayerId,
        killer: Option<PlayerId>,
    },
    // ...
}

#[derive(Component)]
pub struct UiNotifyArea;

#[derive(Component)]
struct NotificationTimeout {
    timer: Timer,
}

fn spawn_notifications(
    mut commands: Commands,
    settings: Res<AllSettings>,
    ass_ui: Res<UiAssets>,
    ass_game: Res<GameAssets>,
    index_players: Res<PlayersIndex>,
    mut evr_notify: EventReader<NotifyEvent>,
    q_notify_area: Query<Entity, With<UiNotifyArea>>,
    q_player_name: Query<&PlayerDisplayName>,
) {
    for ev in evr_notify.read() {
        for e_area in q_notify_area.iter() {
            match ev {
                NotifyEvent::Simple { l10n_heading, l10n_content } => {
                    let e_notify = commands.spawn((
                        ImageBundle {
                            image: UiImage::from(ass_ui.img_9p_notify_simple.clone()),
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(8.0)),
                                margin: UiRect::all(Val::Px(4.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ImageScaleMode::Sliced(TextureSlicer {
                            border: BorderRect::rectangle(4.0, 4.0),
                            ..Default::default()
                        }),
                        NotificationTimeout {
                            timer: Timer::new(Duration::from_millis(5000), TimerMode::Once),
                        },
                    )).id();
                    if let Some(l10n_heading) = l10n_heading {
                        let e_text_heading = commands.spawn((
                            TextBundle {
                                style: Style {
                                    margin: UiRect {
                                        bottom: Val::Px(4.0),
                                        ..UiRect::all(Val::Auto)
                                    },
                                    ..Default::default()
                                },
                                text: Text::from_section(
                                    "",
                                    TextStyle {
                                        font: ass_ui.font_bold.clone(),
                                        font_size: 20.0 * settings.ui.text_scale,
                                        color: Color::WHITE,
                                    }
                                ),
                                ..Default::default()
                            },
                            L10nKey(l10n_heading.clone()),
                        )).id();
                        commands.entity(e_notify).push_children(&[e_text_heading]);
                    }
                    let e_text_content = commands.spawn((
                        TextBundle {
                            text: Text::from_section(
                                "",
                                TextStyle {
                                    font: ass_ui.font.clone(),
                                    font_size: 18.0 * settings.ui.text_scale,
                                    color: Color::WHITE,
                                }
                            ),
                            ..Default::default()
                        },
                        L10nKey(l10n_content.clone()),
                    )).id();
                    commands.entity(e_notify).push_children(&[e_text_content]);
                    commands.entity(e_area).push_children(&[e_notify]);
                },
                NotifyEvent::KillFeed { killed, killer } => {
                    let e_notify = commands.spawn((
                        ImageBundle {
                            image: ass_ui.img_9p_notify_killfeed.clone().into(),
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                padding: UiRect {
                                    left: Val::Px(24.0),
                                    right: Val::Px(24.0),
                                    top: Val::Px(8.0),
                                    bottom: Val::Px(8.0),
                                },
                                margin: UiRect::all(Val::Px(4.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ImageScaleMode::Sliced(TextureSlicer {
                            border: BorderRect::rectangle(7.0, 4.0),
                            ..Default::default()
                        }),
                        NotificationTimeout {
                            timer: Timer::new(Duration::from_millis(5000), TimerMode::Once),
                        },
                    )).id();
                    let e_kill_icon = commands.spawn((
                        AtlasImageBundle {
                            image: ass_game.sprites_img.clone().into(),
                            texture_atlas: TextureAtlas {
                                layout: ass_game.sprites_layout.clone(),
                                index: crate::gfx2d::sprite::EXPLOSION_MINE,
                            },
                            style: Style {
                                width: Val::Px(40.0),
                                height: Val::Px(40.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    )).id();
                    if let Some(killer) = killer {
                        let e_icon_killer = commands.spawn((
                            AtlasImageBundle {
                                background_color: BackgroundColor(
                                    settings.player_colors.visible[killer.i()].into()
                                ),
                                image: ass_game.sprites_img.clone().into(),
                                texture_atlas: TextureAtlas {
                                    layout: ass_game.sprites_layout.clone(),
                                    index: crate::gfx2d::sprite::TILES6 + crate::gfx2d::sprite::TILE_LAND,
                                },
                                style: Style {
                                    width: Val::Px(40.0),
                                    height: Val::Px(40.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                        )).id();
                        let e_subicon_killer = commands.spawn((
                            AtlasImageBundle {
                                image: ass_game.sprites_img.clone().into(),
                                texture_atlas: TextureAtlas {
                                    layout: ass_game.sprites_layout.clone(),
                                    index: crate::gfx2d::sprite::GENT_MINE,
                                },
                                style: Style {
                                    width: Val::Px(40.0),
                                    height: Val::Px(40.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                        )).id();
                        let name_killer = q_player_name
                            .get(index_players.0[killer.i()]).ok()
                            .map(|x| x.0.clone())
                            .unwrap_or_else(|| format!("Player {}", killer.i()));
                        let e_text_killer = commands.spawn((
                            TextBundle {
                                style: Style {
                                    margin: UiRect {
                                        left: Val::Px(8.0),
                                        right: Val::Px(8.0),
                                        ..UiRect::all(Val::Auto)
                                    },
                                    ..Default::default()
                                },
                                text: Text::from_section(
                                    name_killer,
                                    TextStyle {
                                        font: ass_ui.font.clone(),
                                        font_size: 21.0 * settings.ui.text_scale,
                                        color: Color::WHITE,
                                    }
                                ),
                                ..Default::default()
                            },
                        )).id();
                        commands.entity(e_icon_killer).push_children(&[e_subicon_killer]);
                        commands.entity(e_notify).push_children(&[e_icon_killer, e_text_killer]);
                    }
                    let e_icon_killed = commands.spawn((
                        AtlasImageBundle {
                            background_color: BackgroundColor(
                                settings.player_colors.visible[killed.i()].into()
                            ),
                            image: ass_game.sprites_img.clone().into(),
                            texture_atlas: TextureAtlas {
                                layout: ass_game.sprites_layout.clone(),
                                index: crate::gfx2d::sprite::TILES6 + crate::gfx2d::sprite::TILE_DEADSKULL,
                            },
                            style: Style {
                                width: Val::Px(40.0),
                                height: Val::Px(40.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    )).id();
                    let name_killed = q_player_name
                        .get(index_players.0[killed.i()]).ok()
                        .map(|x| x.0.clone())
                        .unwrap_or_else(|| format!("Player {}", killed.i()));
                    let e_text_killed = commands.spawn((
                        TextBundle {
                            style: Style {
                                margin: UiRect {
                                    left: Val::Px(8.0),
                                    right: Val::Px(8.0),
                                    ..UiRect::all(Val::Auto)
                                },
                                ..Default::default()
                            },
                            text: Text::from_section(
                                name_killed,
                                TextStyle {
                                    font: ass_ui.font.clone(),
                                    font_size: 21.0 * settings.ui.text_scale,
                                    color: Color::WHITE,
                                }
                            ),
                            ..Default::default()
                        },
                    )).id();
                    commands.entity(e_notify).push_children(&[e_kill_icon, e_text_killed, e_icon_killed]);
                    commands.entity(e_area).push_children(&[e_notify]);
                },
            }
        }
    }
}

fn notification_timeout(
    mut commands: Commands,
    time: Res<Time>,
    mut q_notification: Query<(Entity, &mut NotificationTimeout)>,
) {
    for (e, mut n_timeout) in q_notification.iter_mut() {
        n_timeout.timer.tick(time.delta());
        if n_timeout.timer.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}
