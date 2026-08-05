use crate::levels::levels::{
    WallBundle, convert_editor_entities, despawn_level_owned, dev_level_keys, rebuild_tile_grid,
    remove_tile_grid, tick_daytime_clock, update_level_ambient,
};
use crate::states::{AppState, IsPaused};
use crate::{
    assets::GameAssets,
    levels::{AnimateAmbient, DayTime},
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            .insert_resource(LevelSelection::index(0))
            .insert_resource(DayTime::default())
            .insert_resource(AnimateAmbient::default())
            .register_ldtk_int_cell::<WallBundle>(1) //1  - wall type in LDtk
            //.add_systems(OnEnter(AppState::InGame), spawn_world)
            .add_systems(
                PostUpdate,
                (despawn_level_owned, convert_editor_entities)
                    .chain()
                    .after(TransformSystems::Propagate)
                    .run_if(resource_exists::<GameAssets>),
            )
            .add_systems(
                Update,
                (
                    rebuild_tile_grid,
                    dev_level_keys,
                    tick_daytime_clock.run_if(in_state(IsPaused::Running)),
                    update_level_ambient
                        .run_if(resource_changed::<DayTime>)
                        .run_if(|a: Res<AnimateAmbient>| a.0),
                ),
            )
            .add_systems(OnExit(AppState::InGame), remove_tile_grid);
    }
}
