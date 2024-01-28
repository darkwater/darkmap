use bevy::prelude::*;
use geo::Point;

#[derive(Component, Reflect)]
pub struct DecorateRequest;

#[derive(Component)]
pub struct WorldPosition(pub Point);
