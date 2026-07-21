use bevy::prelude::*;

#[derive(Message, Debug)]
pub struct CopterCrashed;

#[derive(Message, Debug)]
pub struct PassengerDelivered {
    pub fare: u32,
}

#[derive(Message, Debug)]
pub struct Landed {
    pub at: Vec2,
}

#[derive(Message, Debug)]
pub struct CopterDamaged {
    pub severity: f32,
}

#[derive(Message)]
pub struct Honked {
    pub at: Vec2,
}
