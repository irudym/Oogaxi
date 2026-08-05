use crate::{
    physics::SimSet,
    states::AppState,
    water::water::{simulate_water, splash_on_entry, update_water_mesh},
};
use bevy::prelude::*;

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (simulate_water, splash_on_entry)
                .chain()
                .in_set(SimSet::Contact),
        )
        .add_systems(
            Update,
            (update_water_mesh/* ,animate_water_material*/).run_if(in_state(AppState::InGame)),
        );
    }
}
