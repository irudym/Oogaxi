use bevy::prelude::*;

#[derive(Message, Debug)]
pub struct CopterCrashed;

#[derive(Message, Debug)]
pub struct PassengerDelivered {
    pub fare: u32,
}
