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
    mouse_state: KeyboardMouseInputState,
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

#[derive(Component, Default)]
struct KeyboardMouseInputState {
    to_disambiguate: Vec<(InputActionName, InputAnalogName, Timer, bool)>,
    queue_key_action: Vec<InputActionName>,
    queue_btn_action: Vec<InputActionName>,
    queue_motion: Vec<InputAnalogName>,
    queue_scroll: Vec<InputAnalogName>,
    queue_immediate: Vec<InputActionName>,
    temp_btns: Vec<(Vec<MouseButton>, String)>,
}

type QueryActionActive<'w: 's, 's> = Query<'w, 's, (
    Entity,
    &'w InputActionName,
), (
    With<InputAction>,
    Without<InputAnalog>,
    With<InputActionEnabled>,
    With<InputActionActive>,
)>;
type QueryAnalogActive<'w: 's, 's> = Query<'w, 's, (
    Entity,
    &'w InputAnalogName,
    Has<AnalogSourceMouseMotion>,
    Has<AnalogSourceMouseScroll>,
), (
    With<InputAnalog>,
    Without<InputAction>,
    With<InputAnalogEnabled>,
    With<InputAnalogActive>,
)>;
type QueryActionInactive<'w: 's, 's> = Query<'w, 's, (), (
    With<InputAction>,
    Without<InputAnalog>,
    With<InputActionEnabled>,
    Without<InputActionActive>,
)>;
type QueryAnalogInactive<'w: 's, 's> = Query<'w, 's, (), (
    With<InputAnalog>,
    Without<InputAction>,
    With<InputAnalogEnabled>,
    Without<InputAnalogActive>,
)>;

fn rc_keyboard_mouse_input(
    mut evr_button: EventReader<MouseButtonInput>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_kbd: EventReader<KeyboardInput>,
    q_input: Query<(
        &KeyboardMouseInputState,
    ), (
        With<InputGovernor>,
    )>,
) -> bool {
    let Ok((state,)) = q_input.get_single() else {
        return false;
    };
    !state.to_disambiguate.is_empty() ||
        evr_button.read().count() > 0 ||
        evr_motion.read().count() > 0 ||
        evr_kbd.read().count() > 0
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
        &mut KeyboardMouseInputState,
    ), (
        With<InputGovernor>,
    )>,
    mut q_action_active: QueryActionActive,
    mut q_analog_active: QueryAnalogActive,
    mut q_action_inactive: QueryActionInactive,
    mut q_analog_inactive: QueryAnalogInactive,
) {
    let s_mouse = settings.get::<MouseInputSettings>().unwrap();
    let s_map = settings.get::<KeyboardMouseMappings>().unwrap();
    let (action_map, analog_map, mut state) = q_input.single_mut();

    state.refresh_state_from_inputs(s_map, s_mouse, &in_key, &in_btn);

    state.do_immediate_actions(&mut commands, action_map, &mut q_action_inactive, &mut q_action_active);

    if !evr_motion.is_empty() {
        evr_motion.clear();
        state.do_disambiguate_by_motion(&mut commands, analog_map, &mut q_analog_inactive, &mut q_analog_active);
    }

    state.do_disambiguate_by_timer(&mut commands, action_map, &mut q_action_inactive, time.delta());

    state.do_keyboard_action_activations(&mut commands, action_map, &mut q_action_inactive);
    state.do_mouse_action_activations(&mut commands, action_map, &mut q_action_inactive);
    state.do_mouse_motion_activations(&mut commands, analog_map, &mut q_analog_inactive, &mut q_analog_active);
    state.do_mouse_scroll_activations(&mut commands, analog_map, &mut q_analog_inactive, &mut q_analog_active);

    state.do_action_deactivations(&mut commands, &mut q_action_active);
    state.do_analog_deactivations(&mut commands, &mut q_analog_active);
}

impl KeyboardMouseInputState {
    fn activate_action(commands: &mut Commands, e: Entity, name: &InputActionName) {
        let name = name.clone();
        commands.entity(e).insert(InputActionActive);
        commands.add(move |world: &mut World| {
            world.try_run_schedule(InputActionOnPress(name)).ok();
        });
    }
    fn activate_analog<S: Component>(commands: &mut Commands, e: Entity, name: &InputAnalogName, source: S) {
        let name = name.clone();
        commands.entity(e).insert((InputAnalogActive, source));
        commands.add(move |world: &mut World| {
            world.try_run_schedule(InputAnalogOnStart(name)).ok();
        });
    }
    fn deactivate_action(commands: &mut Commands, e: Entity, name: &InputActionName) {
        let name = name.clone();
        commands.entity(e).remove::<ActionDeactivateCleanup>();
        commands.add(move |world: &mut World| {
            world.try_run_schedule(InputActionOnRelease(name)).ok();
        });
    }
    fn deactivate_analog(commands: &mut Commands, e: Entity, name: &InputAnalogName) {
        let name = name.clone();
        commands.entity(e).remove::<AnalogDeactivateCleanup>();
        commands.add(move |world: &mut World| {
            world.try_run_schedule(InputAnalogOnStop(name)).ok();
        });
    }
    fn do_disambiguate_by_motion(
        &mut self,
        commands: &mut Commands,
        analog_map: &AnalogNameMap,
        q_analog_inactive: &mut QueryAnalogInactive,
        q_analog_active: &mut QueryAnalogActive,
    ) {
        for (_, name, _, ran) in self.to_disambiguate.iter_mut() {
            if *ran { continue; }
            *ran = true;
            if let Some(&e) = analog_map.map_name.get(name) {
                if q_analog_inactive.get(e).is_ok() {
                    debug!("Mouse disambiguated by motion. Activate InputAnalog {:?} (MouseMotion).", name.0);
                    Self::activate_analog(commands, e, name, AnalogSourceMouseMotion);
                }
                if let Ok((_, _, has_motion, _)) = q_analog_active.get(e) {
                    if !has_motion {
                        debug!("Mouse disambiguated by motion. Add MouseMotion to active InputAnalog {:?}.", name.0);
                        Self::activate_analog(commands, e, name, AnalogSourceMouseMotion);
                    }
                }
            }
        }
    }
    fn do_disambiguate_by_timer(
        &mut self,
        commands: &mut Commands,
        action_map: &ActionNameMap,
        q_action_inactive: &mut QueryActionInactive,
        dt: Duration,
    ) {
        for (name, _, timer, ran) in self.to_disambiguate.iter_mut() {
            if *ran { continue; }
            timer.tick(dt);
            if timer.finished() {
                *ran = true;
                if let Some(&e) = action_map.map_name.get(name) {
                    if q_action_inactive.get(e).is_ok() {
                        debug!("Mouse disambiguation timeout. Activate InputAction {:?}.", name.0);
                        Self::activate_action(commands, e, name);
                    }
                }
            }
        }
    }
    fn do_immediate_actions(
        &mut self,
        commands: &mut Commands,
        action_map: &ActionNameMap,
        q_action_inactive: &mut QueryActionInactive,
        q_action_active: &mut QueryActionActive,
    ) {
        for ref name in self.queue_immediate.drain(..) {
            if let Some(&e) = action_map.map_name.get(name) {
                if q_action_inactive.get(e).is_ok() {
                    debug!("Mouse disambiguated. Immediate trigger InputAction {:?}.", name.0);
                    Self::activate_action(commands, e, name);
                    Self::deactivate_action(commands, e, name);
                }
                if q_action_active.get(e).is_ok() {
                    debug!("Mouse disambiguated. Immediate restart InputAction {:?}.", name.0);
                    Self::deactivate_action(commands, e, name);
                    Self::activate_action(commands, e, name);
                }
            }
        }
    }
    fn do_keyboard_action_activations(
        &self,
        commands: &mut Commands,
        action_map: &ActionNameMap,
        q_action_inactive: &mut QueryActionInactive,
    ) {
        for name in self.queue_key_action.iter() {
            if let Some(&e) = action_map.map_name.get(name) {
                if q_action_inactive.get(e).is_ok() {
                    debug!("Activate InputAction {:?} (Keyboard).", name.0);
                    Self::activate_action(commands, e, name);
                }
            }
        }
    }
    fn do_mouse_action_activations(
        &self,
        commands: &mut Commands,
        action_map: &ActionNameMap,
        q_action_inactive: &mut QueryActionInactive,
    ) {
        for name in self.queue_btn_action.iter() {
            if self.to_disambiguate.iter().find(|(n, _, _, _)| n == name).is_some() {
                continue;
            }
            if let Some(&e) = action_map.map_name.get(name) {
                if q_action_inactive.get(e).is_ok() {
                    debug!("Activate InputAction {:?} (MouseButton).", name.0);
                    Self::activate_action(commands, e, name);
                }
            }
        }
    }
    fn do_mouse_motion_activations(
        &self,
        commands: &mut Commands,
        analog_map: &AnalogNameMap,
        q_analog_inactive: &mut QueryAnalogInactive,
        q_analog_active: &mut QueryAnalogActive,
    ) {
        for name in self.queue_motion.iter() {
            if self.to_disambiguate.iter().find(|(_, n, _, _)| n == name).is_some() {
                continue;
            }
            if let Some(&e) = analog_map.map_name.get(name) {
                if q_analog_inactive.get(e).is_ok() {
                    debug!("Activate InputAnalog {:?} (MouseMotion).", name.0);
                    Self::activate_analog(commands, e, name, AnalogSourceMouseMotion);
                }
                if let Ok((_, _, has_motion, _)) = q_analog_active.get(e) {
                    if !has_motion {
                        debug!("Add MouseMotion to active InputAnalog {:?}.", name.0);
                        Self::activate_analog(commands, e, name, AnalogSourceMouseMotion);
                    }
                }
            }
        }
    }
    fn do_mouse_scroll_activations(
        &self,
        commands: &mut Commands,
        analog_map: &AnalogNameMap,
        q_analog_inactive: &mut QueryAnalogInactive,
        q_analog_active: &mut QueryAnalogActive,
    ) {
        for name in self.queue_scroll.iter() {
            if let Some(&e) = analog_map.map_name.get(name) {
                if q_analog_inactive.get(e).is_ok() {
                    debug!("Activate InputAnalog {:?} (MouseScroll).", name.0);
                    Self::activate_analog(commands, e, name, AnalogSourceMouseScroll);
                }
                if let Ok((_, _, _, has_scroll)) = q_analog_active.get(e) {
                    if !has_scroll {
                        debug!("Add MouseScroll to active InputAnalog {:?}.", name.0);
                        Self::activate_analog(commands, e, name, AnalogSourceMouseScroll);
                    }
                }
            }
        }
    }
    fn do_action_deactivations(
        &self,
        commands: &mut Commands,
        q_action_active: &mut QueryActionActive,
    ) {
        for (e, name) in q_action_active.iter() {
            let mut deactivate = true;
            if self.queue_key_action.iter().find(|n| *n == name).is_some() {
                deactivate = false;
            }
            if self.queue_btn_action.iter().find(|n| *n == name).is_some() {
                deactivate = false;
            }
            if deactivate {
                debug!("Deactivate InputAction {:?}.", name.0);
                Self::deactivate_action(commands, e, name);
            }
        }
    }
    fn do_analog_deactivations(
        &self,
        commands: &mut Commands,
        q_analog_active: &mut QueryAnalogActive,
    ) {
        for (e, name, has_motion, has_scroll) in q_analog_active.iter() {
            let mut deactivate = true;
            if self.queue_motion.iter().find(|n| *n == name).is_some() {
                deactivate = false;
            } else if has_motion {
                debug!("Remove MouseMotion from active InputAnalog {:?}.", name.0);
                commands.entity(e).remove::<AnalogSourceMouseMotion>();
            }
            if self.queue_scroll.iter().find(|n| *n == name).is_some() {
                deactivate = false;
            } else if has_scroll {
                debug!("Remove MouseScroll from active InputAnalog {:?}.", name.0);
                commands.entity(e).remove::<AnalogSourceMouseScroll>();
            }
            if deactivate {
                debug!("Deactivate InputAnalog {:?}.", name.0);
                Self::deactivate_analog(commands, e, name);
            }
        }
    }
    fn refresh_state_from_inputs(
        &mut self,
        s_map: &KeyboardMouseMappings,
        s_mouse: &MouseInputSettings,
        in_key: &Res<ButtonInput<KeyCode>>,
        in_btn: &Res<ButtonInput<MouseButton>>,
    ) {
        if in_key.is_changed() || in_btn.is_changed() {
            self.temp_btns.clear();
            self.refresh_key_actions_from_inputs(s_map, in_key);
            self.refresh_btn_actions_from_inputs(s_map, in_key, in_btn);
            self.refresh_mouse_motion_from_inputs(s_map, s_mouse, in_key, in_btn);
            self.refresh_mouse_scroll_from_inputs(s_map, in_key, in_btn);
            self.refresh_process_disambiguations();
            self.temp_btns.clear();
        }
    }
    fn add_ambiguity(
        &mut self,
        s_mouse: &MouseInputSettings,
        action: InputActionName,
        analog: InputAnalogName,
    ) {
        if self.to_disambiguate.iter()
            .find(|(ac, an, _, _)| ac == &action && an == &analog).is_some()
        {
            return;
        }
        let dur = Duration::from_millis(s_mouse.action_motion_disambiguate_ms as u64);
        self.to_disambiguate.push((
            action, analog,
            Timer::new(dur, TimerMode::Once),
            false,
        ));
    }
    fn refresh_key_actions_from_inputs(
        &mut self,
        s_map: &KeyboardMouseMappings,
        in_key: &Res<ButtonInput<KeyCode>>,
    ) {
        self.queue_key_action.clear();
        for (keys, name) in s_map.key_actions.iter() {
            if keys.is_empty() { continue; }
            if keys.iter().all(|key| in_key.pressed(*key)) {
                self.queue_key_action.push(name.into());
            }
        }
    }
    fn refresh_btn_actions_from_inputs(
        &mut self,
        s_map: &KeyboardMouseMappings,
        in_key: &Res<ButtonInput<KeyCode>>,
        in_btn: &Res<ButtonInput<MouseButton>>,
    ) {
        self.queue_btn_action.clear();
        let mut any_keys = false;
        for (keys, map) in s_map.mouse_actions.iter() {
            if keys.is_empty() { continue; }
            if keys.iter().all(|key| in_key.pressed(*key)) {
                for (btns, name) in map.iter() {
                    if btns.is_empty() { continue; }
                    if btns.iter().all(|btn| in_btn.pressed(*btn)) {
                        any_keys = true;
                        self.queue_btn_action.push(name.into());
                        self.temp_btns.push((btns.clone(), name.into()));
                    }
                }
            }
        }
        if !any_keys {
            if let Some(map) = s_map.mouse_actions.get(&vec![]) {
                for (btns, name) in map.iter() {
                    if btns.is_empty() { continue; }
                    if btns.iter().all(|btn| in_btn.pressed(*btn)) {
                        self.queue_btn_action.push(name.into());
                        self.temp_btns.push((btns.clone(), name.into()));
                    }
                }
            }
        }
    }
    fn refresh_mouse_motion_from_inputs(
        &mut self,
        s_map: &KeyboardMouseMappings,
        s_mouse: &MouseInputSettings,
        in_key: &Res<ButtonInput<KeyCode>>,
        in_btn: &Res<ButtonInput<MouseButton>>,
    ) {
        self.queue_motion.clear();
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
                        self.queue_motion.push(name.into());
                        if let Some((_, action)) = self.temp_btns.iter().find(|(b, _)| b == btns) {
                            self.add_ambiguity(s_mouse, action.into(), name.into());
                        }
                    }
                }
                if !any_btns {
                    if let Some(name) = map.get(&vec![]) {
                        any_keys = true;
                        self.queue_motion.push(name.into());
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
                        self.queue_motion.push(name.into());
                        if let Some((_, action)) = self.temp_btns.iter().find(|(b, _)| b == btns) {
                            self.add_ambiguity(s_mouse, action.into(), name.into());
                        }
                    }
                }
                if !any_btns {
                    if let Some(name) = map.get(&vec![]) {
                        self.queue_motion.push(name.into());
                    }
                }
            }
        }
    }
    fn refresh_mouse_scroll_from_inputs(
        &mut self,
        s_map: &KeyboardMouseMappings,
        in_key: &Res<ButtonInput<KeyCode>>,
        in_btn: &Res<ButtonInput<MouseButton>>,
    ) {
        self.queue_scroll.clear();
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
                        self.queue_scroll.push(name.into());
                    }
                }
                if !any_btns {
                    if let Some(name) = map.get(&vec![]) {
                        any_keys = true;
                        self.queue_scroll.push(name.into());
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
                        self.queue_scroll.push(name.into());
                    }
                }
                if !any_btns {
                    if let Some(name) = map.get(&vec![]) {
                        self.queue_scroll.push(name.into());
                    }
                }
            }
        }
    }
    fn refresh_process_disambiguations(&mut self) {
        self.to_disambiguate.retain(|(action, analog, _, _)| {
            let a = self.queue_btn_action.iter().find(|&n| n == action).is_some();
            let m = self.queue_motion.iter().find(|&n| n == analog).is_some();
            match (a, m) {
                (true, true) => { true }, // still ambiguous
                (true, false) => { false }, // no longer ambiguous
                (false, true) => { false }, // no longer ambiguous
                (false, false) => {
                    // completely released; execute action immediately
                    self.queue_immediate.push(action.clone());
                    false
                }
            }
        });
    }
}
