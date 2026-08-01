pub mod GameColors {
    use bevy::prelude::*;
    pub const REST: Color = Color::srgb(0.35, 0.21, 0.13);
    pub const HIGHLIGHT: Color = Color::srgb(0.72, 0.49, 0.17);
    pub const DUST: Color = Color::srgb(0.57, 0.38, 0.23);
    pub const TORCH_CORE: Color = Color::srgb(1.0, 0.64, 0.23);
    pub const TORCH_FLAME: Color = Color::srgb(1.0, 0.82, 0.29);
    pub const WATER: Color = Color::srgb(0.31, 0.7, 0.75);
    pub const WATER_DROPS: Color = Color::srgb(0.87, 0.96, 0.94);

    pub const NIGHT_AMBIENT: Color = Color::srgb(0.1, 0.09, 0.1);
    pub const MORNING_AMBIENT: Color = Color::srgb(0.4, 0.8, 0.9);
    pub const DAY_AMBIENT: Color = Color::srgb(1.0, 1.0, 1.0);
    pub const EVENING_AMBIENT: Color = Color::srgb(0.9, 0.74, 0.54);
}
