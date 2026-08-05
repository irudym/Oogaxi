use crate::animations::animations::{
    animate_meshes, animate_sprites, damage_flash, face_travel_direction, select_copter_clip,
};
use crate::assets::GameAssets;
use crate::integrity::Invulnerable;
use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                select_copter_clip.run_if(resource_exists::<GameAssets>),
                animate_sprites.after(select_copter_clip),
                face_travel_direction,
                damage_flash,
                animate_meshes.run_if(resource_exists::<GameAssets>),
            ),
        );

        app.add_observer(
            |remove: On<Remove, Invulnerable>, mut sprites: Query<&mut Sprite>| {
                if let Ok(mut sprite) = sprites.get_mut(remove.entity) {
                    sprite.color = sprite.color.with_alpha(1.0);
                }
            },
        );
    }
}
