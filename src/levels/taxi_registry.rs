use bevy::prelude::*;

use crate::physics::FlightConfig;

#[derive(Debug)]
pub struct Stop {
    pub address: u8,
    pub sign_pos: Vec2, // captured from TaxiStop entity's GlobalTransform, where passengers wait (sign tile - 1)
    pub cave_pos: Vec2, // captured from CaveEntrance entity's GlobalTransform, where passenger enter and leave the world
}

#[derive(Resource, Default)]
pub struct TaxiRegistry(pub Vec<Stop>);

impl TaxiRegistry {
    pub fn by_address(&self, a: u8) -> Option<&Stop> {
        self.0.iter().find(|s| s.address == a)
    }

    pub fn stop_near(&self, pos: Vec2, radius: f32) -> Option<&Stop> {
        self.0
            .iter()
            .map(|s| (s, s.sign_pos.distance_squared(pos)))
            .filter(|(_, d2)| *d2 <= radius * radius)
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(s, _)| s)
    }

    pub fn fare_between(&self, from: u8, to: u8, cfg: &FlightConfig) -> f32 {
        match (self.by_address(from), self.by_address(to)) {
            (Some(a), Some(b)) => cfg.fare_base + a.sign_pos.distance(b.sign_pos) * cfg.fare_per_px,
            _ => cfg.fare_min,
        }
    }
}
