use bevy::prelude::*;
use rand::{SeedableRng, rngs::StdRng};

#[derive(Resource)]
pub struct GameRng(pub StdRng);
