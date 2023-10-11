use crate::{prelude::*, ui::console::UiConsole, assets::UiAssets};

pub struct TextFieldPlugin;

impl Plugin for TextFieldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextInputFocus>();
        app.add_systems(Update, (
            textfield_focus_on_press
                .in_set(TextInputFocusSet),
            textfield_input
                .in_set(TextInputFocusSet)
                .run_if(rc_text_input)
                .after(textfield_focus_on_press),
            textfield_focus_visual
                .in_set(NeedsSettingsSet)
                .after(TextInputFocusSet),
        ));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextInputFocusSet;

/// Resource to keep track of focused text input entity, if any.
#[derive(Resource, Default)]
pub struct TextInputFocus(Option<Entity>);

/// Marker for entities that can be focused for text input and accept text input.
#[derive(Component)]
struct TextInput {
    e_text: Entity,
}

/// Marker for entities that can be focused for text input and accept text input.
#[derive(Component)]
struct TextFieldText;

fn textfield_focus_on_press(
    mut focus: ResMut<TextInputFocus>,
    mut q_textfield: Query<(
        Entity, &Interaction
    ), (
        Changed<Interaction>, With<TextInput>, Without<UiDisabled>,
    )>,
    mousebtn: Res<Input<MouseButton>>,
) {
    let last = focus.0;
    if mousebtn.just_pressed(MouseButton::Left) {
        focus.bypass_change_detection().0 = None;
    }
    for (e, interaction) in q_textfield.iter_mut() {
        if *interaction == Interaction::Pressed {
            focus.bypass_change_detection().0 = Some(e);
        }
    }
    if focus.0 != last {
        focus.set_changed();
    }
}

fn textfield_focus_visual(
    settings: Res<AllSettings>,
    focus: Res<TextInputFocus>,
    mut q_textfield: Query<(
        Entity, Ref<Interaction>, &mut BackgroundColor, &TextInput,
    ), (
        Without<UiDisabled>,
    )>,
    mut q_text: Query<&mut Text, With<TextFieldText>>,
    mut oldfocus: Local<Option<Entity>>,
) {
    for (e, interaction, mut color, _) in q_textfield.iter_mut() {
        if interaction.is_changed() {
            match *interaction {
                Interaction::Pressed => {
                    *color = BackgroundColor(settings.ui.color_menu_button_selected.into());
                }
                Interaction::Hovered => {
                    *color = BackgroundColor(settings.ui.color_menu_button_selected.into());
                }
                Interaction::None => {
                    if focus.0 != Some(e) {
                        *color = BackgroundColor(settings.ui.color_menu_button.into());
                    }
                }
            }
        }
    }
    if focus.is_changed() {
        // de-focus any previously focused field
        if let Some(oldfocus) = *oldfocus {
            if let Ok((_, _, mut color, textinput)) = q_textfield.get_mut(oldfocus) {
                *color = BackgroundColor(settings.ui.color_menu_button.into());
                if let Ok(mut text) = q_text.get_mut(textinput.e_text) {
                    // hack to keep field thickness/height:
                    //  - delete the cursor (middle part) if field has text
                    //  - make it transparent if there is no other text
                    if text.sections[0].value.is_empty() && text.sections[2].value.is_empty() {
                        text.sections[1].style.color = Color::NONE;
                    } else {
                        text.sections[1].value = "".into();
                    }
                }
            }
        }
        // focus any newly focused field
        if let Some(newfocus) = focus.0 {
            if let Ok((_, _, mut color, textinput)) = q_textfield.get_mut(newfocus) {
                *color = BackgroundColor(settings.ui.color_menu_button_selected.into());
                if let Ok(mut text) = q_text.get_mut(textinput.e_text) {
                    text.sections[1].style.color = settings.ui.color_text.into();
                    text.sections[1].value = "|".into();
                    // for ease of editing, move cursor to end
                    let strtmp = text.sections[2].value.clone();
                    text.sections[0].value.push_str(strtmp.as_str());
                    text.sections[2].value.clear();
                }
            }
        }

        *oldfocus = focus.0;
    }
}

fn textfield_input(
    mut commands: Commands,
    mut focus: ResMut<TextInputFocus>,
    mut evr_char: EventReader<ReceivedCharacter>,
    kbd: Res<Input<KeyCode>>,
    q_textfield: Query<&TextInput>,
    mut q_text: Query<(Entity, &mut Text, &TextInputHandler), With<TextFieldText>>,
) {
    if focus.is_changed() {
        evr_char.clear();
    }

    let Some(e_focus) = focus.0 else {
        return;
    };

    let Ok((e_text, mut text, handler)) = q_textfield
        .get(e_focus)
        .and_then(|f| q_text.get_mut(f.e_text))
    else {
        focus.0 = None;
        evr_char.clear();
        return;
    };

    if kbd.just_pressed(KeyCode::Return) {
        focus.0 = None;
        let mut s = String::new();
        s.push_str(&text.sections[0].value);
        s.push_str(&text.sections[2].value);
        commands.add(move |world: &mut World| {
            TextInputHandler::confirm(world, e_text, s);
        });
        evr_char.clear();
        return;
    }
    if kbd.just_pressed(KeyCode::Escape) {
        focus.0 = None;
        commands.add(move |world: &mut World| {
            TextInputHandler::cancel(world, e_text);
        });
        evr_char.clear();
        return;
    }

    if kbd.just_pressed(KeyCode::Left) {
        if let Some(lastchar) = text.sections[0].value.pop() {
            text.sections[2].value.insert(0, lastchar);
        }
        evr_char.clear();
        return;
    }
    if kbd.just_pressed(KeyCode::Right) {
        if !text.sections[2].value.is_empty() {
            let lastchar = text.sections[2].value.remove(0);
            text.sections[0].value.push(lastchar);
        }
        evr_char.clear();
        return;
    }
    if kbd.just_pressed(KeyCode::Back) {
        text.sections[0].value.pop();
        evr_char.clear();
        return;
    }
    if kbd.just_pressed(KeyCode::Delete) {
        if !text.sections[2].value.is_empty() {
            text.sections[2].value.remove(0);
        }
        evr_char.clear();
        return;
    }

    for ev in evr_char.iter() {
        if ev.char.is_control() {
            continue;
        }
        if !handler.validate(ev.char) {
            continue;
        }
        text.sections[0].value.push(ev.char);
    }
}

fn rc_text_input(
    focus: Option<Res<TextInputFocus>>,
    q_console: Query<(), With<UiConsole>>,
) -> bool {
    focus.map(|f| f.0.is_some()).unwrap_or(false) && q_console.is_empty()
}

pub fn spawn_textfield(
    commands: &mut Commands,
    settings: &AllSettings,
    uiassets: &UiAssets,
    min_width: Val,
    handler: TextInputHandler,
    enabled: bool,
) -> Entity {
    let color_init = if enabled {
        settings.ui.color_menu_button
    } else {
        settings.ui.color_menu_button_inactive
    };
    let color_text = if enabled {
        settings.ui.color_text
    } else {
        settings.ui.color_text_inactive
    };

    let text_style = TextStyle {
        color: color_text.into(),
        font_size: 24.0 * settings.ui.text_scale,
        font: uiassets.font.clone(),
    };

    let text = commands.spawn((
        TextFieldText,
        handler,
        TextBundle {
            text: Text::from_sections([
                TextSection {
                    value: "".into(),
                    style: text_style.clone(),
                },
                TextSection {
                    value: "|".into(),
                    style: TextStyle {
                        color: Color::NONE,
                        font_size: 24.0 * settings.ui.text_scale,
                        font: uiassets.font.clone(),
                    },
                },
                TextSection {
                    value: "".into(),
                    style: text_style.clone(),
                },
            ]),
            ..Default::default()
        },
    )).id();

    let field = commands.spawn((
        TextInput {
            e_text: text,
        },
        Interaction::default(),
        NodeBundle {
            background_color: BackgroundColor(color_init.into()),
            style: Style {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                margin: UiRect::all(Val::Px(4.0)),
                min_width,
                flex_grow: 1.0,
                flex_shrink: 0.0,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    commands.entity(field).push_children(&[text]);

    if !enabled {
        commands.entity(field).insert(UiDisabled);
    }

    commands.add(move |world: &mut World| {
        TextInputHandler::cancel(world, text);
    });

    field
}

#[derive(Component)]
pub struct TextInputHandler {
    validate: Box<dyn Fn(char) -> bool + Send + Sync>,
    confirm: Option<(bool, Box<dyn System<In = String, Out = String>>)>,
    cancel: Option<(bool, Box<dyn System<In = (), Out = String>>)>,
}

impl TextInputHandler {
    pub fn new<MConfirm, MCancel>(
        validate: impl Fn(char) -> bool + Send + Sync + 'static,
        confirm: impl IntoSystem<String, String, MConfirm>,
        cancel: impl IntoSystem<(), String, MCancel>,
    ) -> Self {
        TextInputHandler {
            validate: Box::new(validate),
            confirm: Some((false, Box::new(IntoSystem::into_system(confirm)))),
            cancel: Some((false, Box::new(IntoSystem::into_system(cancel)))),
        }
    }
    fn confirm(world: &mut World, e_self: Entity, text: String) {
        let (initted, mut system) = world.entity_mut(e_self)
            .get_mut::<Self>().unwrap()
            .confirm.take().unwrap();

        if !initted {
            system.initialize(world);
        }
        let r = system.run(text, world);
        system.apply_deferred(world);

        let mut emut = world.entity_mut(e_self);
        emut.get_mut::<Self>().unwrap().confirm = Some((true, system));
        let mut text = emut.get_mut::<Text>().unwrap();
        text.sections[0].value = r;
        text.sections[2].value = "".into();
    }
    fn cancel(world: &mut World, e_self: Entity) {
        let (initted, mut system) = world.entity_mut(e_self)
            .get_mut::<Self>().unwrap()
            .cancel.take().unwrap();

        if !initted {
            system.initialize(world);
        }
        let r = system.run((), world);
        system.apply_deferred(world);

        let mut emut = world.entity_mut(e_self);
        emut.get_mut::<Self>().unwrap().cancel = Some((true, system));
        let mut text = emut.get_mut::<Text>().unwrap();
        text.sections[0].value = r;
        text.sections[2].value = "".into();
    }
    fn validate(&self, c: char) -> bool {
        (self.validate)(c)
    }
}
