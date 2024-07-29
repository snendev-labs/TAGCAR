use avian2d::prelude::{
    Collider, CollisionLayers, FixedJoint, Joint, LayerMask, Mass, RevoluteJoint, RigidBody,
    Sleeping,
};
use bevy::prelude::*;
use bevy_reactive_blueprints::Blueprint;

use crate::Car;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Wheel;

impl Wheel {
    pub const WIDTH: f32 = 8.;
    pub const LENGTH: f32 = 14.;
    pub const MASS: Mass = Mass(10.);
    pub const OFFSET: Vec2 = Vec2 {
        x: Car::LENGTH / 2.1,
        y: Car::WIDTH / 2. + Wheel::WIDTH,
    };
    pub const COLLISION_LAYER: LayerMask = LayerMask(1 << 2);
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Deref, Reflect)]
pub struct PartOfCar(Entity);

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct FrontWheel;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct BackWheel;

#[derive(Bundle)]
pub struct WheelBundle {
    blueprint: Blueprint<Wheel>,
    wheel: Wheel,
    car: PartOfCar,
    rigid_body: RigidBody,
    collider: Collider,
    spatial: SpatialBundle,
    layer: CollisionLayers,
    mass: Mass,
    sleeping: Sleeping,
}

impl WheelBundle {
    pub(crate) fn new(car: Entity, transform: Transform) -> Self {
        let collider = Collider::rectangle(Wheel::LENGTH, Wheel::WIDTH);
        Self {
            blueprint: Blueprint::new(Wheel),
            wheel: Wheel,
            car: PartOfCar(car),
            rigid_body: RigidBody::Dynamic,
            collider,
            spatial: SpatialBundle::from_transform(transform),
            layer: CollisionLayers::new(Wheel::COLLISION_LAYER, LayerMask::NONE),
            mass: Wheel::MASS,
            sleeping: Sleeping,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct WheelJoint;

#[derive(Bundle)]
pub struct FrontWheelJointBundle {
    wheel_joint: WheelJoint,
    joint: RevoluteJoint,
    car: PartOfCar,
}

impl FrontWheelJointBundle {
    pub(crate) fn new(car: Entity, wheel: Entity, offset: Vec2) -> Self {
        FrontWheelJointBundle {
            wheel_joint: WheelJoint,
            joint: RevoluteJoint::new(car, wheel)
                .with_local_anchor_1(offset)
                .with_angle_limits(
                    -Car::MAX_STEERING_DEG.to_radians(),
                    Car::MAX_STEERING_DEG.to_radians(),
                ),
            car: PartOfCar(car),
        }
    }
}

#[derive(Bundle)]
pub struct BackWheelJointBundle {
    wheel_joint: WheelJoint,
    joint: FixedJoint,
    car: PartOfCar,
}

impl BackWheelJointBundle {
    pub(crate) fn new(car: Entity, wheel: Entity, offset: Vec2) -> Self {
        BackWheelJointBundle {
            wheel_joint: WheelJoint,
            joint: FixedJoint::new(car, wheel).with_local_anchor_1(offset),
            car: PartOfCar(car),
        }
    }
}
