use crate::prelude::*;
use crate::assets::UiAssets;
use crate::ui::widget::form::*;
use crate::ui::widget::textfield::{spawn_textfield, TextInputHandler};

use super::*;

pub fn plugin(app: &mut App) {
    app.register_clicommand_noargs("menu_lan_join", spawn_menu_lan_join);
    app.register_clicommand_noargs("menu_lan_setup", spawn_menu_lan_setup);
}

fn spawn_menu_lan_join(
    mut commands: Commands,
    uiassets: Res<UiAssets>,
    settings: Res<AllSettings>,
    mut stack: ResMut<MenuStack>,
    q_container: Query<Entity, With<MenuContainer>>,
    q_extras: Query<Entity, With<MenuTopBarExtras>>,
    mut q_title: Query<&mut L10nKey, With<MenuTitleText>>,
) {
    let Ok(container) = q_container.get_single() else {
        error!("Menu Container Entity not found!");
        return;
    };

    // clear any previous menu
    commands.entity(container).despawn_descendants();
    if let Ok(topbar) = q_extras.get_single() {
        commands.entity(topbar).despawn_descendants();
    }
    if let Ok(mut title) = q_title.get_single_mut() {
        title.0 = "menu-title-lan-join".into();
    }
    // if the previous menu was another LAN menu,
    // replace the entry, else create new entry
    let have_entry = stack.0.last()
        .map(|top| top.starts_with("menu_lan"))
        .unwrap_or(false);
    if have_entry {
        stack.0.pop();
    }
    stack.0.push("menu_lan_join".into());

    // top bar button to switch to lan_setup menu
    if let Ok(topbar) = q_extras.get_single() {
        let butt_lan_setup = spawn_menu_butt(
            &mut commands,
            &*uiassets,
            &*settings,
            OnClick::new().cli("menu_lan_setup"),
            "menu-button-lan-setup",
            "menu-tooltip-lan-setup",
            true,
        );
        commands.entity(topbar).push_children(&[butt_lan_setup]);
    }

    let wrapper = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                min_width: Val::Px(600.),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let field_serverip = spawn_textfield(
        &mut commands, &settings, &uiassets, Val::Px(250.), TextInputHandler::new(
            |_| true,
            move |In(s): In<String>, mut settings: ResMut<AllSettings>| {
                if let Ok(addr) = s.parse() {
                    settings.net.last_host_addr = addr;
                }
                format!("{}", settings.net.last_host_addr)
            },
            move |settings: Res<AllSettings>| {
                format!("{}", settings.net.last_host_addr)
            },
        ), true,
    );
    let field_sessionid = spawn_textfield(
        &mut commands, &settings, &uiassets, Val::Px(80.), TextInputHandler::new(
            |c| c.is_digit(10),
            move |In(s): In<String>, mut settings: ResMut<AllSettings>| {
                if let Ok(sessionid) = s.parse() {
                    settings.net.last_host_sessionid = sessionid;
                }
                format!("{}", settings.net.last_host_sessionid)
            },
            move |settings: Res<AllSettings>| {
                format!("{}", settings.net.last_host_sessionid)
            },
        ), true,
    );

    let form_server_info = create_form_layout(
        &mut commands, &settings, &uiassets, &[
        FormLine("menu-lan-join-label-serverip".into(), field_serverip),
        FormLine("menu-lan-join-label-sessionid".into(), field_sessionid),
    ]);

    let butt_connect = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new().cli("host_connect_last"),
        "menu-button-lan-connect",
        "menu-tooltip-lan-connect",
        true,
    );
    let row_connect = spawn_menu_row(&mut commands, &[butt_connect]);

    commands.entity(wrapper).push_children(&[form_server_info, row_connect]);
    commands.entity(container).push_children(&[wrapper]);
}

fn spawn_menu_lan_setup(
    mut commands: Commands,
    uiassets: Res<UiAssets>,
    settings: Res<AllSettings>,
    mut stack: ResMut<MenuStack>,
    q_container: Query<Entity, With<MenuContainer>>,
    q_extras: Query<Entity, With<MenuTopBarExtras>>,
    mut q_title: Query<&mut L10nKey, With<MenuTitleText>>,
) {
    let Ok(container) = q_container.get_single() else {
        error!("Menu Container Entity not found!");
        return;
    };

    // clear any previous menu
    commands.entity(container).despawn_descendants();
    if let Ok(topbar) = q_extras.get_single() {
        commands.entity(topbar).despawn_descendants();
    }
    if let Ok(mut title) = q_title.get_single_mut() {
        title.0 = "menu-title-lan-setup".into();
    }
    // if the previous menu was another LAN menu,
    // replace the entry, else create new entry
    let have_entry = stack.0.last()
        .map(|top| top.starts_with("menu_lan"))
        .unwrap_or(false);
    if have_entry {
        stack.0.pop();
    }
    stack.0.push("menu_lan_setup".into());

    // top bar button to switch to lan_join menu
    if let Ok(topbar) = q_extras.get_single() {
        let butt_lan_join = spawn_menu_butt(
            &mut commands,
            &*uiassets,
            &*settings,
            OnClick::new().cli("menu_lan_join"),
            "menu-button-lan-join",
            "menu-tooltip-lan-join",
            true,
        );
        commands.entity(topbar).push_children(&[butt_lan_join]);
    }

    let wrapper = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    commands.entity(wrapper).push_children(&[]);
    commands.entity(container).push_children(&[wrapper]);
}
