use bevy::prelude::*;
use std::sync::Arc;

use crate::assets::GameAssets;
use crate::collision::Grounded;
use crate::integrity::Invulnerable;
use crate::materials::FlashMaterial;
use crate::physics::{ThrustInput, Velocity};
use crate::player::Player;

#[derive(Component, Clone)]
pub struct Clip {
    pub frames: Arc<[(usize, f32)]>, // (atlas index, duration in seconds)
    pub looped: bool,
}

impl Clip {
    pub fn same(&self, other: &Clip) -> bool {
        Arc::ptr_eq(&self.frames, &other.frames)
    }

    pub fn once(mut self) -> Self {
        self.looped = false;
        self
    }
}

#[derive(Component, Default)]
pub struct AnimState {
    pub frame: usize, // index into clip, not into atlas
    pub timer: Timer,
}

impl AnimState {
    pub fn reset(&mut self, clip: &Clip) {
        self.frame = 0;
        self.timer = Timer::from_seconds(clip.frames[0].1, TimerMode::Once);
    }
}

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

fn animate_meshes(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut materials: ResMut<Assets<FlashMaterial>>,
    mut query: Query<(&Clip, &mut AnimState, &MeshMaterial2d<FlashMaterial>)>,
) {
    for (clip, mut state, handle) in &mut query {
        state.timer.tick(time.delta());
        if !state.timer.is_finished() {
            continue;
        }
        let last = clip.frames.len() - 1;
        if state.frame < last {
            state.frame += 1;
        } else if clip.looped {
            state.frame = 0;
        }

        let (atlas_index, secs) = clip.frames[state.frame];
        state.timer = Timer::from_seconds(secs, TimerMode::Once);
        if let Some(mut mat) = materials.get_mut(&handle.0) {
            mat.atlas_rect = assets
                .get(crate::assets::Sheet::Copter)
                .atlas_rect(atlas_index);
        }
    }
}

// pull - selection off physics state
fn select_copter_clip(
    assets: Res<GameAssets>,
    mut players: Query<(&mut Clip, &mut AnimState, &ThrustInput, Has<Grounded>), With<Player>>,
) {
    for (mut clip, mut state, thrust, grounded) in &mut players {
        let desired = assets.get(crate::assets::Sheet::Copter).clip(if grounded {
            "idle"
        } else if thrust.vertical > 0.0 {
            "fly"
        } else {
            "fall"
        });
        if !clip.same(&desired) {
            *clip = desired;
            state.reset(&clip);
        }
    }
}

fn animate_sprites(time: Res<Time>, mut query: Query<(&Clip, &mut AnimState, &mut Sprite)>) {
    for (clip, mut state, mut sprite) in &mut query {
        state.timer.tick(time.delta());
        if !state.timer.is_finished() {
            continue;
        }
        let last = clip.frames.len() - 1;
        if state.frame < last {
            state.frame += 1;
        } else if clip.looped {
            state.frame = 0;
        }

        let (atlas_index, secs) = clip.frames[state.frame];
        state.timer = Timer::from_seconds(secs, TimerMode::Once);
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = atlas_index;
        }
    }
}

fn face_travel_direction(mut movers: Query<(&Velocity, &mut Sprite), With<Player>>) {
    for (vel, mut sprite) in &mut movers {
        if vel.x.abs() > 10.0 {
            sprite.flip_x = vel.x < 0.0;
        }
    }
}

/// I-frames made visible - 10Hz square wave off the Virtuial clock, so it freezes correctly on pause
fn damage_blink(time: Res<Time>, mut blinkers: Query<&mut Sprite, With<Invulnerable>>) {
    let visible = (time.elapsed_secs() * 10.0) as u32 % 2 == 0;
    for mut sprite in &mut blinkers {
        sprite.color = sprite.color.with_alpha(if visible { 1.0 } else { 0.25 });
    }
}

fn damage_flash(
    //time: Res<Time>,
    mut materials: ResMut<Assets<FlashMaterial>>,
    flashers: Query<(&MeshMaterial2d<FlashMaterial>, &Invulnerable)>,
) {
    for (handle, invul) in &flashers {
        let Some(mut mat) = materials.get_mut(&handle.0) else {
            return;
        };

        // Sharp spike, quick decay - a flash, not a pulse
        let t = invul.0.fraction();
        // Decay curve:
        // sharp: (1.0 - t * 4.0).max(0.0);
        // soft: (1.0 - t).powi(3)
        mat.amount = (1.0 - t).powi(3);
    }
}
