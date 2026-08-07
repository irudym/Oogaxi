use crate::camera::SceneTexture;
use crate::camera::projection::virtual_projection;
//use crate::layers::WORLD_LAYER;
use crate::physics::Velocity;
use crate::player::Player;
use bevy::camera::RenderTarget;
use bevy::camera::Hdr;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};

use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CameraConfig {
    pub follow_k: f32, // 1/s - half-life = ln(2)/k
    pub lead_time: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        // half-life = 116ms;
        Self {
            follow_k: 6.0,
            lead_time: 0.25,
        }
    }
}

#[derive(Component)]
pub struct GameCamera;

///World-space rect the camera may show, written by levels.rs on level spawn.
#[derive(Resource, Default)]
pub struct LevelBounds {
    pub min: Vec2,
    pub max: Vec2,
}

/// Virtual-resolution projection, kept - now with the ni-bit pipeline
/// (HDR + tonemapping + dither)
pub fn spawn_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let resolution = Vec2::new(640.0, 360.0); // best modes: 640x360, 1280x720

    // create scene texture to apply future shaders
    let scene = images.add(Image::new_target_texture(
        resolution.x as u32,
        resolution.y as u32,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));

    commands.spawn((
        GameCamera,
        Camera2d,
        Hdr,
        Camera {
            order: 0,
            ..default()
        },
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
        Bloom::default(),
        virtual_projection(),
        RenderTarget::Image(scene.clone().into()),
    ));

    commands.insert_resource(SceneTexture(scene));
}

pub fn camera_follow(
    time: Res<Time>,
    config: Res<CameraConfig>,
    bounds: Res<LevelBounds>,
    player: Query<(&Transform, &Velocity), With<Player>>,
    mut camera: Query<(&mut Transform, &Projection), (With<GameCamera>, Without<Player>)>,
) {
    // Query, not Single as the player does not exist during level lead
    // and the dev inspector can add cameras.
    // bypass in case of there is no player -> just focus on the center
    let (player_tf, vel) = match player.single() {
        Ok((tf, v)) => (tf.translation.truncate(), v.0),
        _ => {
            #[cfg(feature = "dev")]
            warn_once!("camera_follow: no player — centering on level");
            (Vec2::ZERO, Vec2::ZERO)
        }
    };

    let Ok((mut cam_tf, projection)) = camera.single_mut() else {
        return;
    };

    let Projection::Orthographic(orth) = projection else {
        return;
    };
    let half_view = orth.area.half_size();

    // 1. where we want to look_ ahead of the player along their motion.
    let target = player_tf + vel * config.lead_time;

    // 2. clamp so the View never leaves the level
    let target = clamp_view(target, half_view, &bounds);

    // 3. frame-rate-independent pursuit - except on a fresh level, where we
    //      SNAP: watching the camera fly over the void from the previous level's
    //      coordinates looks exactly like the bug it would be
    let alpha = if bounds.is_changed() {
        1.0
    } else {
        1.0 - (-config.follow_k * time.delta_secs()).exp()
    };
    let next = cam_tf.translation.truncate().lerp(target, alpha);
    cam_tf.translation.x = next.x;
    cam_tf.translation.y = next.y;
}

/// Clam a camera-center so the viewport stays inside bounds, and when an axis
/// of the level is smaller than the viewport, center that axis instead.
/// One function, two camera modes: "fixed screen" emerges from the math!
fn clamp_view(center: Vec2, half_view: Vec2, b: &LevelBounds) -> Vec2 {
    let mut out = center;
    for axis in 0..2 {
        let lo = b.min[axis] + half_view[axis];
        let hi = b.max[axis] - half_view[axis];
        out[axis] = if lo >= hi {
            (b.min[axis] + b.max[axis]) / 2.0 // level fits: fixed and centered
        } else {
            center[axis].clamp(lo, hi) // level larger: follow, clamped
        };
    }
    out
}

/// Parallax
/// Depth from motion: a layer with factor p (1 = moves with the world, 0 = glued to the camera)
///  sits at layer_pos = camera_pos * (1 − p). Endpoints prove it: p=1 → a normal world object;
/// p=0 → an infinite sky riding the camera; p>1 → foreground, faster than the world

#[derive(Component)]
pub struct ParallaxLayer {
    pub factor: f32, //0 - sky, 0.3 - far cave wall, 0.7 = near rocks
}

pub fn parallax(
    camera: Query<&Transform, (With<GameCamera>, Without<ParallaxLayer>)>,
    mut layers: Query<(&mut Transform, &ParallaxLayer)>,
) {
    let Ok(cam) = camera.single() else { return };
    let cam = cam.translation.truncate();

    for (mut tf, layer) in &mut layers {
        let p = cam * (1.0 - layer.factor);
        tf.translation.x = p.x;
        tf.translation.y = p.y;
    }
}
