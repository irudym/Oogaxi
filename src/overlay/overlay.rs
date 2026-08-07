use bevy::{
    camera::{Hdr, ScalingMode, visibility::RenderLayers},
    prelude::*,
};

use crate::{
    camera::{
        SceneTexture,
        projection::{VIRTUAL_RESOLUTION, virtual_projection},
        refraction_camera::spawn_refraction_camera,
    },
    layers::{OVERLAY_LAYER, REFRACTION_LAYER},
    lights::LightMap,
    materials::{
        LightCompositeMaterial, RefractionPresentMaterial, ScreenMaterial,
        scene_present_material::ScenePresentMaterial,
    },
    z::z,
};

#[derive(Component)]
pub struct OverlayCamera;

pub fn spawn_post_process(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut screen_materials: ResMut<Assets<ScreenMaterial>>,
    mut light_materials: ResMut<Assets<LightCompositeMaterial>>,
    mut scene_materials: ResMut<Assets<ScenePresentMaterial>>,
    mut refraction_materials: ResMut<Assets<RefractionPresentMaterial>>,
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
            order: 3,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        virtual_projection(),
        RenderLayers::layer(OVERLAY_LAYER),
    ));

    let refraction_texture = spawn_refraction_camera(&mut commands, &mut images);

    // create layers
    let quad = meshes.add(Rectangle::from_size(VIRTUAL_RESOLUTION));

    //scene overlay
    commands.spawn((
        Mesh2d(quad.clone()),
        MeshMaterial2d(scene_materials.add(ScenePresentMaterial {
            scene: Some(scene_texture.0.clone()),
        })),
        Transform::from_xyz(0.0, 0.0, z::SCENE),
        RenderLayers::layer(OVERLAY_LAYER), //the scene -> present camera, stationary
    ));

    //refraction overlay
    commands.spawn((
        Mesh2d(quad.clone()),
        MeshMaterial2d(refraction_materials.add(RefractionPresentMaterial {
            refraction: Some(refraction_texture.clone()),
        })),
        Transform::from_xyz(0.0, 0.0, z::REFRACTION),
        RenderLayers::layer(OVERLAY_LAYER), //the scene -> present camera, stationary
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
