use bevy::prelude::*;

#[derive(Message, Debug)]
pub struct CopterCrashed;

#[derive(Message, Debug)]
pub struct PassengerDelivered {
    pub fare: u32,
}

#[derive(Message, Debug)]
pub struct Landed {
    pub platform: Option<Entity>,
}

#[derive(Message, Debug)]
pub struct CopterDamaged {
    pub severity: f32,
}
