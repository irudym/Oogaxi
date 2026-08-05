use bevy::prelude::*;
use std::sync::Arc;

#[derive(Component, Clone)]
pub struct Clip {
    pub frames: Arc<[(usize, f32)]>, // (atlas index, duration in seconds)
    pub looped: bool,
}

impl Clip {
    pub fn same(&self, other: &Clip) -> bool {
        Arc::ptr_eq(&self.frames, &other.frames)
    }

    pub fn once(mut self) -> Self {
        self.looped = false;
        self
    }
}
