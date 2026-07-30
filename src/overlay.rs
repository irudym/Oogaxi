use bevy::{
    camera::{Hdr, visibility::RenderLayers},
    prelude::*,
};

use crate::materials::ScreenMaterial;

const OVERLAY_LAYER: usize = 1;

#[derive(Component)]
pub struct OverlayCamera;

pub fn spawn_post_process(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ScreenMaterial>>,
) {
    commands.spawn((
        Camera2d,
        OverlayCamera,
        Hdr,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(OVERLAY_LAYER),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1280.0, 720.0))),
        MeshMaterial2d(materials.add(ScreenMaterial {
            params: Vec4::new(0.5, 0.05, 0.0, 0.0),
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RenderLayers::layer(OVERLAY_LAYER),
    ));
}
