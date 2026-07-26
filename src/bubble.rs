use bevy::prelude::*;

use crate::{assets::GameAssets, states::IsPaused};

#[derive(Component)]
pub struct Bubble(pub Entity);

#[derive(Component)]
pub struct BubbleTimer(pub Timer);

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

fn update_bubbles(
    mut commands: Commands,
    mut bubbles: Query<(Entity, &mut BubbleTimer, &Bubble)>,
    time: Res<Time>,
) {
    for (entity, mut timer, bubble) in &mut bubbles {
        if timer.0.tick(time.delta()).is_finished() {
            pop_bubble(&mut commands, entity, Some(bubble));
        }
    }
}

pub struct BubblePlugin;

impl Plugin for BubblePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_bubbles.run_if(in_state(IsPaused::Running)));
    }
}
