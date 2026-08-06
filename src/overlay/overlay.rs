use bevy::{
    camera::{Hdr, ScalingMode, visibility::RenderLayers},
    prelude::*,
};

use crate::{
    camera::SceneTexture,
    layers::OVERLAY_LAYER,
    lights::LightMap,
    materials::{
        LightCompositeMaterial, ScreenMaterial, scene_present_material::ScenePresentMaterial,
    },
    z::z,
};

#[derive(Component)]
pub struct OverlayCamera;

pub fn spawn_post_process(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut screen_materials: ResMut<Assets<ScreenMaterial>>,
    mut light_materials: ResMut<Assets<LightCompositeMaterial>>,
    mut scene_materials: ResMut<Assets<ScenePresentMaterial>>,
    light_map: Res<LightMap>,
    scene_texture: Res<SceneTexture>,
) {
    warn!("Create overlay");
    commands.spawn((
        Camera2d,
        OverlayCamera,
        Hdr,
        IsDefaultUiCamera,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Default,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 640.0,
                height: 360.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(OVERLAY_LAYER),
    ));

    // create layers
    let quad = meshes.add(Rectangle::new(640.0, 360.0));

    //scene overlay
    commands.spawn((
        Mesh2d(quad.clone()),
        MeshMaterial2d(scene_materials.add(ScenePresentMaterial {
            scene: Some(scene_texture.0.clone()),
        })),
        Transform::from_xyz(0.0, 0.0, z::SCENE),
        RenderLayers::layer(OVERLAY_LAYER),
    ));

    // lights overlay
    commands.spawn((
        Mesh2d(quad.clone()),
        MeshMaterial2d(light_materials.add(LightCompositeMaterial {
            light_map: Some(light_map.0.clone()),
        })),
        Transform::from_xyz(0.0, 0.0, z::LIGHTS),
        RenderLayers::layer(OVERLAY_LAYER),
    ));

    // effects
    commands.spawn((
        Mesh2d(quad.clone()),
        MeshMaterial2d(screen_materials.add(ScreenMaterial {
            params: Vec4::new(0.5, 0.05, 0.0, 0.0),
        })),
        Transform::from_xyz(0.0, 0.0, z::EFFECTS), // should go after lights
        RenderLayers::layer(OVERLAY_LAYER),
    ));
}
