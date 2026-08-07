use crate::{
    camera::{camera::*, sync_tracking_cameras},
    states::AppState,
};
use bevy::camera::{RenderTarget, visibility::RenderLayers};
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraConfig>()
            .init_resource::<CameraConfig>()
            .init_resource::<LevelBounds>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    camera_follow.run_if(in_state(AppState::InGame)),
                    sync_tracking_cameras.after(camera_follow),
                    parallax.after(camera_follow),
                ),
            );
        #[cfg(feature = "dev")]
        app.add_systems(Update, audit_cameras.run_if(any_camera_changed));
    }
}

#[cfg(feature = "dev")]
fn audit_cameras(
    cameras: Query<(
        Entity,
        &Camera,
        Option<&RenderLayers>,
        Option<&RenderTarget>,
    )>,
) {
    let mut list: Vec<_> = cameras
        .iter()
        .map(|(e, c, l, t)| {
            let target = match t {
                Some(RenderTarget::Image(_)) => "texture",
                Some(RenderTarget::Window(_)) | None => "window",
                _ => "other",
            };
            (c.order, e, format!("{l:?}"), target)
        })
        .collect();
    list.sort_by_key(|(order, _, _, _)| *order);
    info!("--- cameras in the game ({}) ---", list.len());
    for (order, entity, layers, targets) in list {
        info!("camera {entity} order={order} layers={layers:?} target={targets:?}");
    }
}

#[cfg(feature = "dev")]
fn any_camera_changed(
    added: Query<(), Added<Camera>>,
    mut count: Local<usize>,
    all: Query<(), With<Camera>>,
) -> bool {
    let now = all.iter().count();
    let changed = !added.is_empty() || now != *count;
    *count = now;
    changed
}
