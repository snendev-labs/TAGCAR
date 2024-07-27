use avian2d::prelude::{Collider, CollisionLayers, Mass, RigidBody, Sleeping};
use bevy::{ecs::system::StaticSystemParam, prelude::*};

use bevy_reactive_blueprints::{Blueprint, FromBlueprint};

use crate::CarCollisionLayer;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Car;

impl Car {
    pub const WIDTH: f32 = 40.;
    pub const LENGTH: f32 = 60.;
    pub const HEIGHT: f32 = 40.;
    pub const MASS: Mass = Mass(100.);
    pub const ENGINE_POWER: f32 = 1e3;
    pub const REVERSE_POWER: f32 = -8e2;
    pub const MAX_STEERING_DEG: f32 = 18.;
}

#[derive(Clone, Debug, Default)]
#[derive(Bundle)]
pub struct CarBundle {
    pub car: Car,
    pub name: Name,
}

#[derive(Clone, Debug)]
#[derive(Bundle)]
pub struct CarPhysicsBundle {
    rigid_body: RigidBody,
    collider: Collider,
    spatial: SpatialBundle,
    layer: CollisionLayers,
    mass: Mass,
    sleeping: Sleeping,
}

impl CarPhysicsBundle {
    fn from_transform(transform: Transform) -> Self {
        CarPhysicsBundle {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::rectangle(Car::LENGTH, Car::WIDTH),
            spatial: SpatialBundle::from_transform(transform),
            layer: CollisionLayers::new(CarCollisionLayer::Car, [CarCollisionLayer::Car]),
            sleeping: Sleeping,
            mass: Mass(100.),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Reflect)]
pub struct CarBlueprint {
    pub origin: Vec2,
    pub angle: f32,
}

impl CarBlueprint {
    pub fn new(origin: Vec2, angle: f32) -> Blueprint<Self> {
        Blueprint::new(Self { origin, angle })
    }
}

pub(crate) type TotalCarBundle = (CarBundle, CarPhysicsBundle);

impl FromBlueprint<CarBlueprint> for TotalCarBundle {
    type Params<'w, 's> = ();

    fn from_blueprint(
        blueprint: &CarBlueprint,
        _: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        (
            CarBundle {
                car: Car,
                name: Name::new("Car"),
            },
            CarPhysicsBundle::from_transform(
                Transform::from_translation(Vec3::new(blueprint.origin.x, blueprint.origin.y, 30.))
                    .with_rotation(Quat::from_rotation_z(blueprint.angle)),
            ),
        )
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct CarParts {
    pub(crate) wheel_front_left: Entity,
    pub(crate) joint_front_left: Entity,
    pub(crate) wheel_front_right: Entity,
    pub(crate) joint_front_right: Entity,
    pub(crate) wheel_back_left: Entity,
    pub(crate) joint_back_left: Entity,
    pub(crate) wheel_back_right: Entity,
    pub(crate) joint_back_right: Entity,
}