use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    /// Held: upward thrust
    Thrust,

    /// Axis: horizontal steer, -1.0 .. =1.0
    #[actionlike(Axis)]
    Move,
    Honk,
    Pause,
}

/// one player's binding - keyboard and gamepad in a single map
pub fn input_map_for(player: u8) -> InputMap<Action> {
    let keyboard = match player {
        1 => InputMap::default()
            .with(Action::Thrust, KeyCode::ArrowUp)
            .with(Action::Thrust, KeyCode::Space)
            .with_axis(Action::Move, VirtualAxis::horizontal_arrow_keys())
            .with(Action::Honk, KeyCode::Enter)
            .with(Action::Pause, KeyCode::Escape),
        _ => InputMap::default()
            .with(Action::Thrust, KeyCode::KeyW)
            .with_axis(Action::Move, VirtualAxis::ad())
            .with(Action::Honk, KeyCode::KeyQ)
            .with(Action::Pause, KeyCode::Escape),
    };

    // Gamepad bindings are identical for every player - separate PADS, not separate scheme
    keyboard
        .with(Action::Thrust, GamepadButton::South)
        .with(Action::Thrust, GamepadButton::RightTrigger2)
        .with_axis(
            Action::Move,
            GamepadControlAxis::LEFT_X.with_deadzone_symmetric(0.1),
        )
        .with(Action::Honk, GamepadButton::West)
        .with(Action::Pause, GamepadButton::Start)
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());
    }
}
