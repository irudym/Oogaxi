pub mod z {
    pub const SKY: f32 = -10.0;
    pub const PARALLAX_FAR: f32 = -8.0;
    pub const PARALLAX_NEAR: f32 = -6.0;
    // bevy_ecs_ldtk tile layers land in small positive values near 0–2:
    pub const PASSENGER: f32 = 4.0;
    pub const HAZARD: f32 = 6.0;
    pub const PLAYER: f32 = 10.0;
    pub const FX: f32 = 15.0;
    pub const LIGHTS: f32 = 9.0;
}
