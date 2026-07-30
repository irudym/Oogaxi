use bevy::prelude::*;

use crate::{colors::GameColors, states::AppState, z::z};

fn emissive(color: Color, strength: f32) -> Color {
    let c = color.to_linear();
    Color::LinearRgba(LinearRgba {
        red: c.red * strength,
        green: c.green * strength,
        blue: c.blue * strength,
        alpha: c.alpha,
    })
}

#[derive(Component)]
struct Torch;

pub fn spawn_torch(commands: &mut Commands, pos: Vec2) {
    commands.spawn((
        Torch,
        Sprite::from_color(emissive(GameColors::TORCH_CORE, 5.0), Vec2::splat(4.0)),
        Transform::from_translation(pos.extend(z::LIGHTS)),
        DespawnOnExit(AppState::InGame),
    ));
}
