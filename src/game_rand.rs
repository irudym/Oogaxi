use bevy::prelude::*;
use rand::rngs::StdRng;

#[derive(Resource)]
pub struct GameRng(pub StdRng);
