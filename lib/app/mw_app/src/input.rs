use bevy::input::{gamepad::GamepadEvent, keyboard::KeyboardInput, mouse::{MouseButtonInput, MouseMotion, MouseWheel}, ButtonState};
use mw_app_core::input::*;

use crate::{prelude::*, settings::{KeyboardMouseMappings, MouseInputSettings}};

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
        OnKeyboardEventSet
            .run_if(on_event::<KeyboardInput>()),
        OnMouseButtonEventSet
            .run_if(on_event::<MouseButtonInput>()),
        OnMouseScrollEventSet
            .run_if(on_event::<MouseWheel>()),
        OnMouseMotionEventSet
            .run_if(rc_on_mouse_motion_or_cursor_event),
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
    app.add_systems(
        OnExit(AppState::InGame),
        (
            remove_from_all::<ActionDisableCleanup, With<InputAction>>,
            remove_from_all::<AnalogDisableCleanup, With<InputAnalog>>,
        )
    );
    app.add_systems(Update, (
        detect_input_devices
            .run_if(rc_detect_input_devices)
            .in_set(SetStage::Provide(GameInputSS::Detect)),
        keyboard_mouse_input
            .run_if(rc_keyboard_mouse_input)
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
    core: InputGovernorCoreBundle,
    action_name_map: ActionNameMap,
    analog_name_map: AnalogNameMap,
    mouse_state: MouseState,
}

#[derive(Component, Default)]
pub struct ActionNameMap {
    pub map_name: HashMap<InputActionName, Entity>,
    pub map_entity: HashMap<Entity, InputActionName>,
}

#[derive(Component, Default)]
pub struct AnalogNameMap {
    pub map_name: HashMap<InputAnalogName, Entity>,
    pub map_entity: HashMap<Entity, InputAnalogName>,
}

#[derive(Component, Default)]
struct MouseState {
    to_disambiguate: Vec<(InputActionName, InputAnalogName, Timer, bool)>,
}

#[derive(Bundle)]
pub struct ActionDeactivateCleanup {
    active: InputActionActive,
}

#[derive(Bundle)]
pub struct AnalogDeactivateCleanup {
    active: InputAnalogActive,
    sources: AnalogSourcesCleanup,
}

#[derive(Bundle)]
pub struct ActionDisableCleanup {
    enabled: InputActionEnabled,
    active: InputActionActive,
}

#[derive(Bundle)]
pub struct AnalogDisableCleanup {
    enabled: InputAnalogEnabled,
    active: InputAnalogActive,
    sources: AnalogSourcesCleanup,
}

fn rc_on_mouse_motion_or_cursor_event(
    mut evr_mouse: EventReader<MouseMotion>,
    mut evr_cursor: EventReader<CursorMoved>,
) -> bool {
    evr_mouse.read().count() > 0 ||
    evr_cursor.read().count() > 0
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
    mut evr_mouse: EventReader<MouseMotion>,
    mut evr_kbd: EventReader<KeyboardInput>,
    mut evr_touch: EventReader<TouchInput>,
    mut evr_gamepad: EventReader<GamepadEvent>,
) -> bool {
    evr_mouse.read().count() > 0 ||
    evr_kbd.read().count() > 0 ||
    evr_touch.read().count() > 0 ||
    evr_gamepad.read().count() > 0
}

fn detect_input_devices(
    mut evr_mouse: EventReader<MouseMotion>,
    mut evr_kbd: EventReader<KeyboardInput>,
    mut evr_touch: EventReader<TouchInput>,
    mut evr_gamepad: EventReader<GamepadEvent>,
    gamepads: Res<Gamepads>,
    mut q_input: Query<&mut DetectedInputDevices, With<InputGovernor>>,
) {
    let mut detected = q_input.single_mut();
    if !detected.mouse && evr_mouse.read().count() > 0 {
        detected.mouse = true;
    }
    if !detected.kbd && evr_kbd.read().count() > 0 {
        detected.kbd = true;
    }
    if !detected.touch && evr_touch.read().count() > 0 {
        detected.touch = true;
    }
    if evr_gamepad.read().count() > 0 {
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
        action_name_map.map_name.insert(name.clone(), e);
        action_name_map.map_entity.insert(e, name.clone());
    }

    for e in removed_analog.read() {
        if let Some(name) = analog_name_map.map_entity.remove(&e) {
            analog_name_map.map_name.remove(&name);
        }
    }
    for (e, name) in &q_analog {
        analog_name_map.map_name.insert(name.clone(), e);
        analog_name_map.map_entity.insert(e, name.clone());
    }
}

fn rc_keyboard_mouse_input(
    mut evr_button: EventReader<MouseButtonInput>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_kbd: EventReader<KeyboardInput>,
    q_input: Query<(
        &MouseState,
    ), (
        With<InputGovernor>,
    )>,
) -> bool {
    let r = evr_button.read().count() > 0 ||
        evr_motion.read().count() > 0 ||
        evr_kbd.read().count() > 0;
    let Ok((state,)) = q_input.get_single() else {
        return r;
    };
    r || !state.to_disambiguate.is_empty()
}

fn keyboard_mouse_input(
    mut commands: Commands,
    settings: Settings,
    time: Res<Time>,
    mut evr_motion: EventReader<MouseMotion>,
    in_key: Res<ButtonInput<KeyCode>>,
    in_btn: Res<ButtonInput<MouseButton>>,
    mut q_input: Query<(
        &ActionNameMap,
        &AnalogNameMap,
        &mut MouseState,
    ), (
        With<InputGovernor>,
    )>,
    q_action_active: Query<(
        Entity,
        &InputActionName,
    ), (
        With<InputAction>,
        Without<InputAnalog>,
        With<InputActionEnabled>,
        With<InputActionActive>,
    )>,
    q_analog_active: Query<(
        Entity,
        &InputAnalogName,
        Has<AnalogSourceMouseMotion>,
        Has<AnalogSourceMouseScroll>,
    ), (
        With<InputAnalog>,
        Without<InputAction>,
        With<InputAnalogEnabled>,
        With<InputAnalogActive>,
    )>,
    q_action_inactive: Query<(), (
        With<InputAction>,
        Without<InputAnalog>,
        With<InputActionEnabled>,
        Without<InputActionActive>,
    )>,
    q_analog_inactive: Query<(), (
        With<InputAnalog>,
        Without<InputAction>,
        With<InputAnalogEnabled>,
        Without<InputAnalogActive>,
    )>,
    mut queue_key_action: Local<Vec<InputActionName>>,
    mut queue_btn_action: Local<Vec<InputActionName>>,
    mut queue_motion: Local<Vec<InputAnalogName>>,
    mut queue_scroll: Local<Vec<InputAnalogName>>,
    mut queue_immediate: Local<Vec<InputActionName>>,
) {
    let s_mouse = settings.get::<MouseInputSettings>().unwrap();
    let s_map = settings.get::<KeyboardMouseMappings>().unwrap();
    let (action_map, analog_map, mut state) = q_input.single_mut();

    if in_key.is_changed() || in_btn.is_changed() {
        let mut btns_triggered: Vec<(Vec<MouseButton>, String)> = default();
        queue_key_action.clear();
        queue_btn_action.clear();
        queue_motion.clear();
        queue_scroll.clear();
        for (keys, name) in s_map.key_actions.iter() {
            if keys.is_empty() { continue; }
            if keys.iter().all(|key| in_key.pressed(*key)) {
                queue_key_action.push(name.into());
            }
        }
        let mut any_keys = false;
        for (keys, map) in s_map.mouse_actions.iter() {
            if keys.is_empty() { continue; }
            if keys.iter().all(|key| in_key.pressed(*key)) {
                for (btns, name) in map.iter() {
                    if btns.is_empty() { continue; }
                    if btns.iter().all(|btn| in_btn.pressed(*btn)) {
                        any_keys = true;
                        queue_btn_action.push(name.into());
                        btns_triggered.push((btns.clone(), name.into()));
                    }
                }
            }
        }
        if !any_keys {
            if let Some(map) = s_map.mouse_actions.get(&vec![]) {
                for (btns, name) in map.iter() {
                    if btns.is_empty() { continue; }
                    if btns.iter().all(|btn| in_btn.pressed(*btn)) {
                        queue_btn_action.push(name.into());
                        btns_triggered.push((btns.clone(), name.into()));
                    }
                }
            }
        }
        let mut any_keys = false;
        for (keys, map) in s_map.mouse_motion.iter() {
            if keys.is_empty() { continue; }
            if keys.iter().all(|key| in_key.pressed(*key)) {
                let mut any_btns = false;
                for (btns, name) in map.iter() {
                    if btns.is_empty() { continue; }
                    if btns.iter().all(|btn| in_btn.pressed(*btn)) {
                        any_keys = true;
                        any_btns = true;
                        queue_motion.push(name.into());
                        if let Some((_, action)) = btns_triggered.iter().find(|(b, _)| b == btns) {
                            let dur = Duration::from_millis(s_mouse.action_motion_disambiguate_ms as u64);
                            state.to_disambiguate.push((
                                action.into(),
                                name.into(),
                                Timer::new(dur, TimerMode::Once),
                                false,
                            ));
                        }
                    }
                }
                if !any_btns {
                    if let Some(name) = map.get(&vec![]) {
                        any_keys = true;
                        queue_motion.push(name.into());
                    }
                }
            }
        }
        if !any_keys {
            if let Some(map) = s_map.mouse_motion.get(&vec![]) {
                let mut any_btns = false;
                for (btns, name) in map.iter() {
                    if btns.is_empty() { continue; }
                    if btns.iter().all(|btn| in_btn.pressed(*btn)) {
                        any_btns = true;
                        queue_motion.push(name.into());
                        if let Some((_, action)) = btns_triggered.iter().find(|(b, _)| b == btns) {
                            let dur = Duration::from_millis(s_mouse.action_motion_disambiguate_ms as u64);
                            state.to_disambiguate.push((
                                action.into(),
                                name.into(),
                                Timer::new(dur, TimerMode::Once),
                                false,
                            ));
                        }
                    }
                }
                if !any_btns {
                    if let Some(name) = map.get(&vec![]) {
                        queue_motion.push(name.into());
                    }
                }
            }
        }
        let mut any_keys = false;
        for (keys, map) in s_map.mouse_scroll.iter() {
            if keys.is_empty() { continue; }
            if keys.iter().all(|key| in_key.pressed(*key)) {
                let mut any_btns = false;
                for (btns, name) in map.iter() {
                    if btns.is_empty() { continue; }
                    if btns.iter().all(|btn| in_btn.pressed(*btn)) {
                        any_keys = true;
                        any_btns = true;
                        queue_scroll.push(name.into());
                    }
                }
                if !any_btns {
                    if let Some(name) = map.get(&vec![]) {
                        any_keys = true;
                        queue_scroll.push(name.into());
                    }
                }
            }
        }
        if !any_keys {
            if let Some(map) = s_map.mouse_scroll.get(&vec![]) {
                let mut any_btns = false;
                for (btns, name) in map.iter() {
                    if btns.is_empty() { continue; }
                    if btns.iter().all(|btn| in_btn.pressed(*btn)) {
                        any_btns = true;
                        queue_scroll.push(name.into());
                    }
                }
                if !any_btns {
                    if let Some(name) = map.get(&vec![]) {
                        queue_scroll.push(name.into());
                    }
                }
            }
        }

        // check for disambiguations that were released
        state.to_disambiguate.retain(|(action, analog, _, _)| {
            let a = queue_btn_action.iter().find(|&n| n == action).is_some();
            let m = queue_motion.iter().find(|&n| n == analog).is_some();
            match (a, m) {
                (true, true) => { true }, // still ambiguous
                (true, false) => { false }, // no longer ambiguous
                (false, true) => { false }, // no longer ambiguous
                (false, false) => {
                    // no longer ambiguous; execute action immediately
                    queue_immediate.push(action.clone());
                    false
                }
            }
        });
    }

    if !evr_motion.is_empty() {
        evr_motion.clear();
        for (_, name, _, ran) in state.to_disambiguate.iter_mut() {
            if *ran { continue; }
            *ran = true;
            if let Some(e) = analog_map.map_name.get(name) {
                if q_analog_active.get(*e).is_ok() {
                    debug!("Mouse disambiguated by motion. Activate InputAnalog {:?} (MouseMotion).", name.0);
                    let name = name.clone();
                    commands.entity(*e).insert((
                        InputAnalogActive,
                        AnalogSourceMouseMotion,
                    ));
                    commands.add(move |world: &mut World| {
                        world.try_run_schedule(InputAnalogOnStart(name)).ok();
                    });
                }
                if let Ok((_, _, has_motion, _)) = q_analog_active.get(*e) {
                    if !has_motion {
                        debug!("Mouse disambiguated by motion. Add MouseMotion to active InputAnalog {:?}.", name.0);
                        let name = name.clone();
                        commands.entity(*e).insert(AnalogSourceMouseMotion);
                        commands.add(move |world: &mut World| {
                            world.try_run_schedule(InputAnalogOnStart(name)).ok();
                        });
                    }
                }
            }
        }
    }

    for (name, _, timer, ran) in state.to_disambiguate.iter_mut() {
        if *ran { continue; }
        timer.tick(time.delta());
        if timer.finished() {
            *ran = true;
            if let Some(e) = action_map.map_name.get(name) {
                if q_action_active.get(*e).is_ok() {
                    debug!("Mouse disambiguation timeout. Activate InputAction {:?}.", name.0);
                    let name = name.clone();
                    commands.entity(*e)
                        .insert(InputActionActive);
                    commands.add(move |world: &mut World| {
                        world.try_run_schedule(InputActionOnPress(name)).ok();
                    });
                }
            }
        }
    }

    for name in queue_immediate.drain(..) {
        if let Some(e) = action_map.map_name.get(&name) {
            if q_action_inactive.get(*e).is_ok() {
                debug!("Mouse disambiguated. Immediate InputAction {:?}.", name.0);
                let n = name.clone();
                commands.add(move |world: &mut World| {
                    world.try_run_schedule(InputActionOnPress(n)).ok();
                });
                let n = name.clone();
                commands.add(move |world: &mut World| {
                    world.try_run_schedule(InputActionOnRelease(n)).ok();
                });
            }
        }
    }

    for name in queue_key_action.iter() {
        if let Some(e) = action_map.map_name.get(name) {
            if q_action_inactive.get(*e).is_ok() {
                debug!("Activate InputAction {:?} (Keyboard).", name.0);
                let name = name.clone();
                commands.entity(*e)
                    .insert(InputActionActive);
                commands.add(move |world: &mut World| {
                    world.try_run_schedule(InputActionOnPress(name)).ok();
                });
            }
        }
    }

    for name in queue_btn_action.iter() {
        if state.to_disambiguate.iter().find(|(n, _, _, _)| n == name).is_some() {
            continue;
        }
        if let Some(e) = action_map.map_name.get(name) {
            if q_action_inactive.get(*e).is_ok() {
                debug!("Activate InputAction {:?} (MouseButton).", name.0);
                let name = name.clone();
                commands.entity(*e)
                    .insert(InputActionActive);
                commands.add(move |world: &mut World| {
                    world.try_run_schedule(InputActionOnPress(name)).ok();
                });
            }
        }
    }

    for name in queue_motion.iter() {
        if state.to_disambiguate.iter().find(|(_, n, _, _)| n == name).is_some() {
            continue;
        }
        if let Some(e) = analog_map.map_name.get(name) {
            if q_analog_inactive.get(*e).is_ok() {
                debug!("Activate InputAnalog {:?} (MouseMotion).", name.0);
                let name = name.clone();
                commands.entity(*e).insert((
                    InputAnalogActive,
                    AnalogSourceMouseMotion,
                ));
                commands.add(move |world: &mut World| {
                    world.try_run_schedule(InputAnalogOnStart(name)).ok();
                });
            }
            if let Ok((_, _, has_motion, _)) = q_analog_active.get(*e) {
                if !has_motion {
                    debug!("Add MouseMotion to active InputAnalog {:?}.", name.0);
                    let name = name.clone();
                    commands.entity(*e).insert(AnalogSourceMouseMotion);
                    commands.add(move |world: &mut World| {
                        world.try_run_schedule(InputAnalogOnStart(name)).ok();
                    });
                }
            }
        }
    }

    for name in queue_scroll.iter() {
        if let Some(e) = analog_map.map_name.get(name) {
            if q_analog_inactive.get(*e).is_ok() {
                debug!("Activate InputAnalog {:?} (MouseScroll).", name.0);
                let name = name.clone();
                commands.entity(*e).insert((
                    InputAnalogActive,
                    AnalogSourceMouseScroll,
                ));
                commands.add(move |world: &mut World| {
                    world.try_run_schedule(InputAnalogOnStart(name)).ok();
                });
            }
            if let Ok((_, _, _, has_scroll)) = q_analog_active.get(*e) {
                if !has_scroll {
                    debug!("Add MouseScroll to active InputAnalog {:?}.", name.0);
                    let name = name.clone();
                    commands.entity(*e).insert(AnalogSourceMouseScroll);
                    commands.add(move |world: &mut World| {
                        world.try_run_schedule(InputAnalogOnStart(name)).ok();
                    });
                }
            }
        }
    }

    for (e, name) in q_action_active.iter() {
        let mut deactivate = true;
        if queue_key_action.iter().find(|n| *n == name).is_some() {
            deactivate = false;
        }
        if queue_btn_action.iter().find(|n| *n == name).is_some() {
            deactivate = false;
        }
        if deactivate {
            debug!("Deactivate InputAction {:?}.", name.0);
            let name = name.clone();
            commands.entity(e)
                .remove::<ActionDeactivateCleanup>();
            commands.add(move |world: &mut World| {
                world.try_run_schedule(InputActionOnRelease(name)).ok();
            });
        }
    }

    for (e, name, has_motion, has_scroll) in q_analog_active.iter() {
        let mut deactivate = true;
        if queue_motion.iter().find(|n| *n == name).is_some() {
            deactivate = false;
        } else if has_motion {
            debug!("Remove MouseMotion from active InputAnalog {:?}.", name.0);
            commands.entity(e)
                .remove::<AnalogSourceMouseMotion>();
        }
        if queue_scroll.iter().find(|n| *n == name).is_some() {
            deactivate = false;
        } else if has_scroll {
            debug!("Remove MouseScroll from active InputAnalog {:?}.", name.0);
            commands.entity(e)
                .remove::<AnalogSourceMouseScroll>();
        }
        if deactivate {
            debug!("Deactivate InputAnalog {:?}.", name.0);
            let name = name.clone();
            commands.entity(e)
                .remove::<AnalogDeactivateCleanup>();
            commands.add(move |world: &mut World| {
                world.try_run_schedule(InputAnalogOnStop(name)).ok();
            });
        }
    }
}
