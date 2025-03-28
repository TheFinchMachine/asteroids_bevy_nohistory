use crate::control::PlayerController;
use crate::schedule::InGameSet;
use crate::input_actions::*;
use bevy::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy::input::gamepad::{Gamepad, GamepadButton};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<InputConfig>::new(&["input.ron"]));
        app.insert_resource(LoadInput(true));
        app.add_event::<InputEvent>();
        app.add_systems(Update, (handle_player_input,).in_set(InGameSet::GameInput));
        app.add_systems(Update, (build_input_map_when_loaded)
            .run_if(load_input)
            .in_set(InGameSet::LoadEntities));
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let handle = asset_server.load("a.input.ron");
    commands.insert_resource(InputConfigHandle(handle));
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum InputBinding {
    KeyboardPressed(KeyCode),
    KeyboardJustPressed(KeyCode),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
    GamepadAxis(GamepadAxis),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Binding {
    binding: InputBinding,
    input: Input,
}

#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
struct InputConfig {
    bindings: Vec<Binding>,
}

#[derive(Resource)]
struct InputConfigHandle(Handle<InputConfig>);

type ActionFn = Box<dyn Fn(Entity) -> InputEvent + Send + Sync>;

// Resource wrapper
#[derive(Resource)]
struct InputMap(HashMap<InputBinding, ActionFn>);

fn build_input_map_from_config(config: InputConfig) -> InputMap {
    let mut map: HashMap<InputBinding, ActionFn> = HashMap::new();

    for b in config.bindings {
        map.insert(b.binding, Box::new(move |controller| InputEvent {
            controller,
            input: b.input,
        }));
    }
    InputMap(map)
}

#[derive(Resource)]
struct LoadInput(bool);

fn load_input(load: Res<LoadInput>) -> bool {
    load.0
}

fn build_input_map_when_loaded(
    mut commands: Commands,
    config_handle: Res<InputConfigHandle>,
    configs: Res<Assets<InputConfig>>,
    mut load: ResMut<LoadInput>,
) {
    if let Some(config) = configs.get(&config_handle.0) {
        let map = build_input_map_from_config(config.clone());
        commands.insert_resource(map);

        // only load once
        load.0 = false;
    }
}

fn handle_player_input(
    controllers: Query<Entity, With<PlayerController>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mouse: Res<ButtonInput<MouseButton>>,
    input_map: Option<Res<InputMap>>,
    mut writer: EventWriter<InputEvent>,
) {
    let input_map = match input_map {
        Some(map) => map,
        None => return,
    };

    for entity in controllers.iter() {
        for (binding, action_fn) in &input_map.0 {
            match *binding {
                InputBinding::KeyboardPressed(code) if keyboard.pressed(code) => {
                    writer.send(action_fn(entity));
                }
                InputBinding::KeyboardJustPressed(code) if keyboard.just_pressed(code) => {
                    writer.send(action_fn(entity));
                }
                InputBinding::GamepadButton(code) => {
                    for gamepad in gamepads.iter() {
                        if gamepad.pressed(code) {
                            writer.send(action_fn(entity));
                            break;
                        }
                    }
                }
                InputBinding::MouseButton(code) if mouse.pressed(code) => {
                    writer.send(action_fn(entity));
                }
                InputBinding::GamepadAxis(axis) => {
                    for gamepad in gamepads.iter() {
                        if let Some(axis_value) = gamepad.get(axis) {
                            let mut event = action_fn(entity);
                            event.input.value = axis_value * event.input.value;
                            writer.send(event);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
