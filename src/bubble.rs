use bevy::prelude::*;

use crate::assets::GameAssets;

#[derive(Component)]
pub struct Bubble(pub Entity);

pub fn spawn_bubble(
    commands: &mut Commands,
    assets: &GameAssets,
    parent: Entity,
    glyph: usize,
) -> Entity {
    commands
        .spawn((
            Sprite {
                image: assets.bubble.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: assets.bubble.layout.clone(),
                    index: 0, // glyph,
                }),
                ..default()
            },
            Transform::from_xyz(18.0, 18.0, 0.5), // relative to parent
            ChildOf(parent),
        ))
        .id()
}

pub fn pop_bubble(commands: &mut Commands, entity: Entity, bubble: Option<&Bubble>) {
    if let Some(b) = bubble {
        commands.entity(b.0).despawn();
        commands.entity(entity).remove::<Bubble>();
    }
}
