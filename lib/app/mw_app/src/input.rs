use bevy::input::{gamepad::GamepadEvent, keyboard::KeyboardInput, mouse::{MouseButtonInput, MouseMotion}, ButtonState};
use mw_app_core::input::*;

use crate::{prelude::*, settings::MouseInputSettings};

pub fn plugin(app: &mut App) {
    app.configure_stage_set(
        Update, GameInputSS::Detect,
        any_filter::<(With<InputGovernor>, Changed<DetectedInputDevices>)>
    );
    app.configure_stage_set_no_rc(
        Update, GameInputSS::Handle,
    );
    app.configure_stage_set_no_rc(
        Startup, GameInputSS::Setup,
    );
    app.configure_sets(Update, (
        GameInputSet
            .run_if(rc_accepting_game_input),
    ));
    app.configure_sets(Update, (
        InputDeviceSet::Keyboard
            .in_set(SetStage::Want(GameInputSS::Detect))
            .run_if(rc_input_device(InputDeviceSet::Keyboard)),
        InputDeviceSet::Mouse
            .in_set(SetStage::Want(GameInputSS::Detect))
            .run_if(rc_input_device(InputDeviceSet::Mouse)),
        InputDeviceSet::Touch
            .in_set(SetStage::Want(GameInputSS::Detect))
            .run_if(rc_input_device(InputDeviceSet::Touch)),
        InputDeviceSet::Gamepad
            .in_set(SetStage::Want(GameInputSS::Detect))
            .run_if(rc_input_device(InputDeviceSet::Gamepad)),
    ));
    app.add_systems(Update, (
        detect_input_devices
            .run_if(rc_detect_input_devices)
            .in_set(SetStage::Provide(GameInputSS::Detect)),
        (
            (
                keyboard_actions,
                keyboard_analogs,
            )
                .in_set(InputDeviceSet::Keyboard)
                .run_if(on_event::<KeyboardInput>()),
            (
                mouse_input,
            )
                .in_set(InputDeviceSet::Mouse)
                .run_if(rc_mouse_input),
        )
            .in_set(GameInputSet)
            .in_set(SetStage::Provide(GameInputSS::Handle)),
    ));
    app.add_systems(Startup, (
        setup_input
            .in_set(SetStage::Prepare(GameInputSS::Setup)),
        manage_inputs
            .in_set(SetStage::Want(GameInputSS::Setup))
            .in_set(SetStage::Prepare(SettingsSyncSS)),
    ));
}

fn setup_input(mut commands: Commands) {
    commands.spawn(InputGovernorBundle::default());
}

#[derive(Bundle, Default)]
pub struct InputGovernorBundle {
    pub core: InputGovernorCoreBundle,
    pub action_name_map: ActionNameMap,
    pub analog_name_map: AnalogNameMap,
    pub key_action_map: KeyActionMap,
    pub key_analog_map: KeyAnalogMap,
    pub mouse_map: MouseMap,
    pub mouse_to_disambiguate: MouseToDisambiguate,
}

#[derive(Component, Default)]
pub struct ActionNameMap {
    pub map_name: HashMap<String, Entity>,
    pub map_entity: HashMap<Entity, String>,
}

#[derive(Component, Default)]
pub struct AnalogNameMap {
    pub map_name: HashMap<String, Entity>,
    pub map_entity: HashMap<Entity, String>,
}

#[derive(Component, Default)]
pub struct KeyActionMap {
    pub map_key: HashMap<KeyCode, Entity>,
    pub map_entity: HashMap<Entity, KeyCode>,
}

#[derive(Component, Default)]
pub struct KeyAnalogMap {
    pub motion_key: HashMap<KeyCode, Entity>,
    pub motion_entity: HashMap<Entity, KeyCode>,
    pub scroll_key: HashMap<KeyCode, Entity>,
    pub scroll_entity: HashMap<Entity, KeyCode>,
}

#[derive(Component, Default)]
pub struct MouseMap {
    pub action_btn: HashMap<MouseButton, Entity>,
    pub action_entity: HashMap<Entity, MouseButton>,
    pub motion_btn: HashMap<MouseButton, Entity>,
    pub motion_entity: HashMap<Entity, MouseButton>,
}

fn rc_accepting_game_input(
    q_inhibit: Query<(), With<InhibitGameInput>>,
    q_ui_interaction: Query<&Interaction>,
) -> bool {
    if !q_inhibit.is_empty() {
        return false;
    }
    let any_interaction = q_ui_interaction.iter().any(|i| *i != Interaction::None);
    !any_interaction
}

fn rc_input_device(device: InputDeviceSet)
    -> impl Fn(Query<&DetectedInputDevices, With<InputGovernor>>) -> bool
{
    match device {
        InputDeviceSet::Keyboard =>
            |q: Query<&DetectedInputDevices, With<InputGovernor>>|
            q.get_single().map(|d| d.kbd)
                .unwrap_or(false),
        InputDeviceSet::Mouse =>
            |q: Query<&DetectedInputDevices, With<InputGovernor>>|
            q.get_single().map(|d| d.mouse)
                .unwrap_or(false),
        InputDeviceSet::Touch =>
            |q: Query<&DetectedInputDevices, With<InputGovernor>>|
            q.get_single().map(|d| d.touch)
                .unwrap_or(false),
        InputDeviceSet::Gamepad =>
            |q: Query<&DetectedInputDevices, With<InputGovernor>>|
            q.get_single().map(|d| d.gamepad)
                .unwrap_or(false),
    }
}

fn rc_detect_input_devices(
    evr_mouse: Res<Events<MouseMotion>>,
    evr_kbd: Res<Events<KeyboardInput>>,
    evr_touch: Res<Events<TouchInput>>,
    evr_gamepad: Res<Events<GamepadEvent>>,
) -> bool {
    !evr_mouse.is_empty() ||
    !evr_kbd.is_empty() ||
    !evr_touch.is_empty() ||
    !evr_gamepad.is_empty()
}

fn detect_input_devices(
    evr_mouse: Res<Events<MouseMotion>>,
    evr_kbd: Res<Events<KeyboardInput>>,
    evr_touch: Res<Events<TouchInput>>,
    evr_gamepad: Res<Events<GamepadEvent>>,
    gamepads: Res<Gamepads>,
    mut q_input: Query<&mut DetectedInputDevices, With<InputGovernor>>,
) {
    let mut detected = q_input.single_mut();
    if !detected.mouse && !evr_mouse.is_empty() {
        detected.mouse = true;
    }
    if !detected.kbd && !evr_kbd.is_empty() {
        detected.kbd = true;
    }
    if !detected.touch && !evr_touch.is_empty() {
        detected.touch = true;
    }
    if !evr_gamepad.is_empty() {
        detected.gamepad = gamepads.iter().count() != 0;
    }
}

fn manage_inputs(
    mut q_input: Query<(
        &mut ActionNameMap,
        &mut AnalogNameMap,
    ), With<InputGovernor>>,
    q_action: Query<(Entity, &InputActionName), Added<InputAction>>,
    q_analog: Query<(Entity, &InputAnalogName), Added<InputAnalog>>,
    mut removed_action: RemovedComponents<InputAction>,
    mut removed_analog: RemovedComponents<InputAnalog>,
) {
    let (
        mut action_name_map,
        mut analog_name_map,
    ) = q_input.single_mut();

    for e in removed_action.read() {
        if let Some(name) = action_name_map.map_entity.remove(&e) {
            action_name_map.map_name.remove(&name);
        }
    }
    for (e, name) in &q_action {
        action_name_map.map_name.insert(name.0.clone(), e);
        action_name_map.map_entity.insert(e, name.0.clone());
    }

    for e in removed_analog.read() {
        if let Some(name) = analog_name_map.map_entity.remove(&e) {
            analog_name_map.map_name.remove(&name);
        }
    }
    for (e, name) in &q_analog {
        analog_name_map.map_name.insert(name.0.clone(), e);
        analog_name_map.map_entity.insert(e, name.0.clone());
    }
}

fn keyboard_actions(
    mut commands: Commands,
    mut evr: EventReader<KeyboardInput>,
    q_input: Query<(
        &KeyActionMap,
    ), (
        With<InputGovernor>,
    )>,
    q_action: Query<(
        &InputActionName,
    ), (
        With<InputAction>,
        With<InputActionEnabled>,
    )>,
) {
    let (key_action_map,) = q_input.single();
    for ev in evr.read() {
        let Some(&e) = key_action_map.map_key.get(&ev.key_code) else {
            continue;
        };
        let Ok((name,)) = q_action.get(e) else {
            continue;
        };
        let name = name.clone();
        match ev.state {
            ButtonState::Pressed => {
                commands.entity(e)
                    .insert(InputActionActive);
                commands.add(move |world: &mut World| {
                    world.try_run_schedule(InputActionOnPress(name)).ok();
                });
            }
            ButtonState::Released => {
                commands.entity(e)
                    .remove::<InputActionActive>();
                commands.add(move |world: &mut World| {
                    world.try_run_schedule(InputActionOnRelease(name)).ok();
                });
            }
        }
    }
}

fn keyboard_analogs(
    mut commands: Commands,
    mut evr: EventReader<KeyboardInput>,
    q_input: Query<(
        &KeyAnalogMap,
    ), (
        With<InputGovernor>,
    )>,
    q_analog: Query<(
        &InputAnalogName,
    ), (
        With<InputAnalog>,
        With<InputAnalogEnabled>,
    )>,
) {
    let (key_analog_map,) = q_input.single();
    for ev in evr.read() {
        if let Some(&e) = key_analog_map.motion_key.get(&ev.key_code) {
            if let Ok((name,)) = q_analog.get(e) {
                let name = name.clone();
                match ev.state {
                    ButtonState::Pressed => {
                        commands.entity(e).insert(AnalogSourceMouseMotion);
                        commands.add(move |world: &mut World| {
                            world.try_run_schedule(InputAnalogOnStart(name)).ok();
                        });
                    }
                    ButtonState::Released => {
                        commands.entity(e)
                            .remove::<AnalogSourceMouseMotion>();
                        commands.add(move |world: &mut World| {
                            world.try_run_schedule(InputAnalogOnStop(name)).ok();
                        });
                    }
                }
            }
        }
        if let Some(&e) = key_analog_map.scroll_key.get(&ev.key_code) {
            if let Ok((name,)) = q_analog.get(e) {
                let name = name.clone();
                match ev.state {
                    ButtonState::Pressed => {
                        commands.entity(e).insert(AnalogSourceMouseScroll);
                        commands.add(move |world: &mut World| {
                            world.try_run_schedule(InputAnalogOnStart(name)).ok();
                        });
                    }
                    ButtonState::Released => {
                        commands.entity(e)
                            .remove::<AnalogSourceMouseScroll>();
                        commands.add(move |world: &mut World| {
                            world.try_run_schedule(InputAnalogOnStop(name)).ok();
                        });
                    }
                }
            }
        }
    }
}

#[derive(Component, Default)]
pub struct MouseToDisambiguate(HashMap<MouseButton, Timer>);

fn rc_mouse_input(
    evr_button: EventReader<MouseButtonInput>,
    evr_motion: EventReader<MouseMotion>,
    q_input: Query<(
        &MouseToDisambiguate,
    ), (
        With<InputGovernor>,
    )>,
) -> bool {
    let r = !evr_button.is_empty() || !evr_motion.is_empty();
    let Ok((to_disambiguate,)) = q_input.get_single() else {
        return r;
    };
    r || !to_disambiguate.0.is_empty()
}

fn mouse_input(
    mut commands: Commands,
    settings: Settings,
    time: Res<Time>,
    mut evr_button: EventReader<MouseButtonInput>,
    mut evr_motion: EventReader<MouseMotion>,
    mut q_input: Query<(
        &MouseMap,
        &mut MouseToDisambiguate,
    ), (
        With<InputGovernor>,
    )>,
    q_action: Query<(
        &InputActionName,
    ), (
        With<InputAction>,
        With<InputActionEnabled>,
    )>,
    q_analog: Query<(
        &InputAnalogName,
    ), (
        With<InputAnalog>,
        With<InputAnalogEnabled>,
    )>,
) {
    let s_mouse = settings.get::<MouseInputSettings>().unwrap();
    let (mouse_map, mut to_disambiguate) = q_input.single_mut();

    for ev in evr_button.read() {
        let e_action = mouse_map.action_btn.get(&ev.button);
        let e_motion = mouse_map.motion_btn.get(&ev.button);
        match (e_action, e_motion) {
            (None, None) => continue,
            (Some(&e_action), Some(&e_motion)) => {
                match ev.state {
                    ButtonState::Pressed => {
                        // ambiguous
                        let dur = Duration::from_millis(s_mouse.action_motion_disambiguate_ms as u64);
                        to_disambiguate.0.insert(ev.button, Timer::new(dur, TimerMode::Once));
                    }
                    ButtonState::Released => {
                        // if there is a pending disambiguation,
                        // immediately process it as action
                        if to_disambiguate.0.remove(&ev.button).is_some() {
                            let Ok((name,)) = q_action.get(e_action) else {
                                continue;
                            };
                            let name = name.clone();
                            commands.add(move |world: &mut World| {
                                world.try_run_schedule(InputActionOnPress(name.clone())).ok();
                                world.try_run_schedule(InputActionOnRelease(name)).ok();
                            });
                        }
                    }
                }
            },
            (Some(&e_action), None) => {
                let Ok((name,)) = q_action.get(e_action) else {
                    continue;
                };
                let name = name.clone();
                match ev.state {
                    ButtonState::Pressed => {
                        commands.entity(e_action)
                            .insert(InputActionActive);
                        commands.add(move |world: &mut World| {
                            world.try_run_schedule(InputActionOnPress(name)).ok();
                        });
                    }
                    ButtonState::Released => {
                        commands.entity(e_action)
                            .remove::<InputActionActive>();
                        commands.add(move |world: &mut World| {
                            world.try_run_schedule(InputActionOnRelease(name)).ok();
                        });
                    }
                }
            },
            (None, Some(&e_motion)) => {
                let Ok((name,)) = q_analog.get(e_motion) else {
                    continue;
                };
                let name = name.clone();
                match ev.state {
                    ButtonState::Pressed => {
                        commands.entity(e_motion).insert(AnalogSourceMouseMotion);
                        commands.add(move |world: &mut World| {
                            world.try_run_schedule(InputAnalogOnStart(name)).ok();
                        });
                    }
                    ButtonState::Released => {
                        commands.entity(e_motion)
                            .remove::<AnalogSourceMouseMotion>();
                        commands.add(move |world: &mut World| {
                            world.try_run_schedule(InputAnalogOnStop(name)).ok();
                        });
                    }
                }
            },
        };
    }

    // all un-disambiguated buttons turn into analogs
    // immediately as soon as any motion is detected
    if !evr_motion.is_empty() {
        evr_motion.clear();
        for (btn, _) in to_disambiguate.0.drain() {
            let Some(&e) = mouse_map.motion_btn.get(&btn) else {
                continue;
            };
            commands.entity(e).insert(AnalogSourceMouseMotion);
            let Ok((name,)) = q_analog.get(e) else {
                continue;
            };
            let name = name.clone();
            commands.add(move |world: &mut World| {
                world.try_run_schedule(InputAnalogOnStart(name)).ok();
            });
        }
    }

    for (btn, timer) in to_disambiguate.0.iter_mut() {
        timer.tick(time.delta());
    }

    for (btn, _) in to_disambiguate.0.extract_if(|_, timer| timer.finished()) {
        // disambiguate as action
        let Some(&e) = mouse_map.action_btn.get(&btn) else {
            continue;
        };
        commands.entity(e)
            .insert(InputActionActive);
        let Ok((name,)) = q_action.get(e) else {
            continue;
        };
        let name = name.clone();
        commands.add(move |world: &mut World| {
            world.try_run_schedule(InputActionOnPress(name)).ok();
        });
    }
}
