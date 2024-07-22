use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Component)]
pub struct Car;

pub struct CarPhysicsBundle {
    rigid_body: RigidBody,
    collider: Collider,
}

pub struct CarGraphicsBundle {
    pbr: PbrBundle,
}
