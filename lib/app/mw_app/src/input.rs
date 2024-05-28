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
        (
            (
                keyboard_actions
                    .in_set(InputDeviceSet::Keyboard)
                    .in_set(OnKeyboardEventSet),
                mouse_input
                    .in_set(InputDeviceSet::Mouse)
                    .run_if(rc_mouse_input),
            ),
            (
                resolve_actions,
                resolve_analogs,
            ),
        ).chain()
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
    mouse_to_disambiguate: MouseState,
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
pub struct MouseState {
    to_disambiguate: HashMap<MouseButton, (Timer, InputActionName, InputAnalogName)>,
    held_buttons: HashSet<MouseButton>,
}

#[derive(Bundle, Default)]
pub struct ActionExtrasBundle {
    activation: ActivationStack,
    wants_immediate: WantsImmediate,
}

#[derive(Bundle, Default)]
pub struct AnalogExtrasBundle {
    activation: ActivationStack,
    wants_analog: WantsAnalog,
}

#[derive(Component, Default)]
struct ActivationStack(HashSet<ActivationSource>);

#[derive(Component, Default)]
struct WantsImmediate(bool);

#[derive(Component, Default)]
struct WantsAnalog(Vec<AnalogSource>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ActivationSource {
    IdleDefault,
    Key(KeyCode),
    MouseButton(MouseButton),
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
    mut commands: Commands,
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
        commands.entity(e).insert(ActionExtrasBundle::default());
    }

    for e in removed_analog.read() {
        if let Some(name) = analog_name_map.map_entity.remove(&e) {
            analog_name_map.map_name.remove(&name);
        }
    }
    for (e, name) in &q_analog {
        analog_name_map.map_name.insert(name.clone(), e);
        analog_name_map.map_entity.insert(e, name.clone());
        commands.entity(e).insert(AnalogExtrasBundle::default());
    }
}

fn resolve_actions(
    mut commands: Commands,
    mut q_action: Query<(
        Entity,
        &ActivationStack,
        &mut WantsImmediate,
        &InputActionName,
        Has<InputActionActive>,
    ), (
        With<InputAction>,
        Without<InputAnalog>,
        With<InputActionEnabled>,
        Or<(
            Changed<ActivationStack>,
            Changed<WantsImmediate>,
        )>,
    )>,
) {
    for (e, stack, mut wants_immediate, name, active) in &mut q_action {
        if wants_immediate.0 {
            let n = name.clone();
            if active {
                commands.entity(e)
                    .remove::<ActionDeactivateCleanup>();
                commands.add(move |world: &mut World| {
                    world.try_run_schedule(InputActionOnRelease(n)).ok();
                });
            }
            let n = name.clone();
            commands.entity(e)
                .insert(InputActionActive);
            commands.add(move |world: &mut World| {
                world.try_run_schedule(InputActionOnPress(n)).ok();
            });
            let n = name.clone();
            commands.entity(e)
                .remove::<ActionDeactivateCleanup>();
            commands.add(move |world: &mut World| {
                world.try_run_schedule(InputActionOnRelease(n)).ok();
            });
        }
        if stack.0.is_empty() && (active && !wants_immediate.0) {
            let name = name.clone();
            commands.entity(e)
                .remove::<ActionDeactivateCleanup>();
            commands.add(move |world: &mut World| {
                world.try_run_schedule(InputActionOnRelease(name)).ok();
            });
        }
        if !stack.0.is_empty() && (!active || wants_immediate.0) {
            let name = name.clone();
            commands.entity(e)
                .insert(InputActionActive);
            commands.add(move |world: &mut World| {
                world.try_run_schedule(InputActionOnPress(name)).ok();
            });
        }
        wants_immediate.0 = false;
    }
}

fn resolve_analogs(
    mut commands: Commands,
    mut q_analog: Query<(
        Entity,
        &ActivationStack,
        &mut WantsAnalog,
        &InputAnalogName,
        Has<InputAnalogActive>,
    ), (
        With<InputAnalog>,
        Without<InputAction>,
        With<InputAnalogEnabled>,
        Or<(
            Changed<ActivationStack>,
            Changed<WantsAnalog>,
        )>,
    )>,
) {
    for (e, stack, mut wants_analog, name, active) in &mut q_analog {
        if stack.0.is_empty() && active {
            let name = name.clone();
            commands.entity(e)
                .remove::<AnalogDeactivateCleanup>();
            commands.add(move |world: &mut World| {
                world.try_run_schedule(InputAnalogOnStop(name)).ok();
            });
        }
        if !stack.0.is_empty() && !active {
            let name = name.clone();
            for analog in wants_analog.0.drain(..) {
                match analog {
                    AnalogSource::MouseMotion(x) => {
                        commands.entity(e).insert(x);
                    },
                    AnalogSource::MouseScroll(x) => {
                        commands.entity(e).insert(x);
                    },
                    AnalogSource::GamepadStick(x) => {
                        commands.entity(e).insert(x);
                    },
                    AnalogSource::GamepadZ(x) => {
                        commands.entity(e).insert(x);
                    },
                }
            }
            commands.entity(e)
                .insert(InputAnalogActive);
            commands.add(move |world: &mut World| {
                world.try_run_schedule(InputAnalogOnStart(name)).ok();
            });
        }
    }
}

fn keyboard_actions(
    settings: Settings,
    mut evr: EventReader<KeyboardInput>,
    q_input: Query<(
        &ActionNameMap,
    ), (
        With<InputGovernor>,
    )>,
    mut q_action: Query<(
        &mut ActivationStack,
    ), (
        With<InputAction>,
        With<InputActionEnabled>,
    )>,
) {
    let s_map = settings.get::<KeyboardMouseMappings>().unwrap();
    let (action_map,) = q_input.single();
    for ev in evr.read() {
        if let Some((mut stack,)) = s_map.key_actions.get(&ev.key_code)
            .and_then(|name| action_map.map_name.get(name))
            .and_then(|e| q_action.get_mut(*e).ok())
        {
            match ev.state {
                ButtonState::Pressed => {
                    stack.0.insert(ActivationSource::Key(ev.key_code));
                }
                ButtonState::Released => {
                    stack.0.remove(&ActivationSource::Key(ev.key_code));
                }
            }
        }
    }
}

fn rc_mouse_input(
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

fn mouse_input(
    settings: Settings,
    time: Res<Time>,
    mut evr_button: EventReader<MouseButtonInput>,
    mut evr_motion: EventReader<MouseMotion>,
    evr_kbd: EventReader<KeyboardInput>,
    in_kbd: Res<ButtonInput<KeyCode>>,
    mut q_input: Query<(
        &ActionNameMap,
        &AnalogNameMap,
        &mut MouseState,
    ), (
        With<InputGovernor>,
    )>,
    mut q: ParamSet<(
        Query<(
            &mut ActivationStack,
            &mut WantsImmediate,
        ), (
            With<InputAction>,
            With<InputActionEnabled>,
        )>,
        Query<(
            &mut ActivationStack,
            &mut WantsAnalog,
        ), (
            With<InputAnalog>,
            With<InputAnalogEnabled>,
        )>,
        Query<(
            &mut ActivationStack,
        ), (
            Or<(
                (
                    With<InputAction>,
                    With<InputActionEnabled>,
                    With<InputActionActive>,
                ),
                (
                    With<InputAnalog>,
                    With<InputAnalogEnabled>,
                    With<InputAnalogActive>,
                ),
            )>,
        )>,
    )>,
) {
    let s_mouse = settings.get::<MouseInputSettings>().unwrap();
    let s_map = settings.get::<KeyboardMouseMappings>().unwrap();
    let (action_map, analog_map, mut state) = q_input.single_mut();
    for ev in evr_button.read() {
        match ev.state {
            ButtonState::Pressed => {
                let action = s_map.get_mouse_action(&in_kbd, ev.button);
                let motion = s_map.get_mouse_motion(&in_kbd, Some(ev.button));
                let scroll = s_map.get_mouse_scroll(&in_kbd, Some(ev.button));

                if action.is_none() && motion.is_none() && scroll.is_none() {
                    continue;
                }
                state.held_buttons.insert(ev.button);

                // scroll always works, no need to disambiguate
                if let Some(scroll) = scroll {
                    if let Some(e) = analog_map.map_name.get(scroll) {
                        if let Ok((mut stack, mut wants_analog)) = q.p1().get_mut(*e) {
                            stack.0.insert(ActivationSource::MouseButton(ev.button));
                            wants_analog.0.push(AnalogSource::MouseScroll(AnalogSourceMouseScroll));
                        }
                    }
                }

                match (action, motion) {
                    (None, None) => {},
                    (Some(action), Some(motion)) => {
                        let dur = Duration::from_millis(s_mouse.action_motion_disambiguate_ms as u64);
                        state.to_disambiguate.insert(ev.button, (
                            Timer::new(dur, TimerMode::Once),
                            action.clone(), motion.clone(),
                        ));
                    },
                    (Some(action), None) => {
                        if let Some(e) = action_map.map_name.get(action) {
                            if let Ok((mut stack, _)) = q.p0().get_mut(*e) {
                                stack.0.insert(ActivationSource::MouseButton(ev.button));
                            }
                        }
                    },
                    (None, Some(motion)) => {
                        if let Some(e) = analog_map.map_name.get(motion) {
                            if let Ok((mut stack, mut wants_analog)) = q.p1().get_mut(*e) {
                                stack.0.insert(ActivationSource::MouseButton(ev.button));
                                wants_analog.0.push(AnalogSource::MouseMotion(AnalogSourceMouseMotion));
                            }
                        }
                    },
                }
            }
            ButtonState::Released => {
                // iterate over everything to stop anything that
                // may have been started regardless of modifier key
                for (mut stack,) in &mut q.p2() {
                    if stack.0.contains(&ActivationSource::MouseButton(ev.button)) {
                        stack.0.remove(&ActivationSource::MouseButton(ev.button));
                    }
                }
                if let Some((_, action, _)) = state.to_disambiguate.remove(&ev.button) {
                    // if there is a pending disambiguation,
                    // immediately process it as action
                    if let Some(e) = action_map.map_name.get(&action) {
                        if let Ok((_, mut immediate)) = q.p0().get_mut(*e) {
                            immediate.0 = true;
                        }
                    }
                }
                state.held_buttons.remove(&ev.button);
            }
        }
    }

    // activate/deactivate idle defaults as necessary
    if !evr_button.is_empty() || !evr_kbd.is_empty() {
        for (mut stack,) in &mut q.p2() {
            if stack.0.contains(&ActivationSource::IdleDefault) {
                stack.0.remove(&ActivationSource::IdleDefault);
            }
        }
        if state.held_buttons.is_empty() {
            // enable default analogs
            let motion = s_map.get_mouse_motion(&in_kbd, None);
            let scroll = s_map.get_mouse_scroll(&in_kbd, None);
            if let Some(motion) = motion {
                if let Some(e) = analog_map.map_name.get(motion) {
                    if let Ok((mut stack, mut wants_analog)) = q.p1().get_mut(*e) {
                        stack.0.insert(ActivationSource::IdleDefault);
                        wants_analog.0.push(AnalogSource::MouseMotion(AnalogSourceMouseMotion));
                    }
                }
            }
            if let Some(scroll) = scroll {
                if let Some(e) = analog_map.map_name.get(scroll) {
                    if let Ok((mut stack, mut wants_analog)) = q.p1().get_mut(*e) {
                        stack.0.insert(ActivationSource::IdleDefault);
                        wants_analog.0.push(AnalogSource::MouseScroll(AnalogSourceMouseScroll));
                    }
                }
            }
        }
    }

    // all un-disambiguated buttons turn into analogs
    // immediately as soon as any motion is detected
    if !evr_motion.is_empty() {
        evr_motion.clear();
        for (btn, (_, _, motion)) in state.to_disambiguate.drain() {
            if let Some(e) = analog_map.map_name.get(&motion) {
                if let Ok((mut stack, mut wants_analog)) = q.p1().get_mut(*e) {
                    stack.0.insert(ActivationSource::MouseButton(btn));
                    wants_analog.0.push(AnalogSource::MouseMotion(AnalogSourceMouseMotion));
                }
            }
        }
    }

    for (btn, (timer, _, _)) in state.to_disambiguate.iter_mut() {
        timer.tick(time.delta());
    }

    for (btn, (_, action, _)) in state.to_disambiguate.extract_if(|_, (timer, _, _)| timer.finished()) {
        // disambiguate as action
        if let Some(e) = action_map.map_name.get(&action) {
            if let Ok((mut stack, _)) = q.p0().get_mut(*e) {
                stack.0.insert(ActivationSource::MouseButton(btn));
            }
        }
    }
}
