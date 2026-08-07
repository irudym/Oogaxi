use bevy::{camera::ScalingMode, prelude::*};

pub const VIRTUAL_RESOLUTION: Vec2 = Vec2::new(640.0, 360.0);

pub fn virtual_projection() -> Projection {
    Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::Fixed {
            width: VIRTUAL_RESOLUTION.x,
            height: VIRTUAL_RESOLUTION.y,
        },
        ..OrthographicProjection::default_2d()
    })
}
