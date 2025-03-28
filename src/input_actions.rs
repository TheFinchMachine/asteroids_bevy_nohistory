use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Actions {
    MoveForward,
    Shoot,
    Rotate,
    Pause,
    Restart,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Input {
    pub action: Actions,
    pub value: f32,
}

#[derive(Event)]
pub struct InputEvent {
    pub controller: Entity,
    pub input: Input
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
