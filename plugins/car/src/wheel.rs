use avian2d::prelude::{
    Collider, CollisionLayers, FixedJoint, Joint, LayerMask, Mass, RevoluteJoint, RigidBody,
    Sleeping,
};
use bevy::prelude::*;

use crate::Car;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Wheel;

impl Wheel {
    pub const COLLISION_LAYER: u8 = 2;
    pub const WIDTH: f32 = 5.;
    pub const LENGTH: f32 = 10.;
    pub const MASS: Mass = Mass(10.);
    pub const OFFSET: Vec2 = Vec2 {
        x: Car::LENGTH / 2.1,
        y: Car::WIDTH / 2. + Wheel::WIDTH,
    };
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct FrontWheel;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct BackWheel;

#[derive(Bundle)]
pub struct WheelBundle {
    wheel: Wheel,
    rigid_body: RigidBody,
    collider: Collider,
    spatial: SpatialBundle,
    layer: CollisionLayers,
    mass: Mass,
    sleeping: Sleeping,
}

impl WheelBundle {
    pub(crate) fn new(transform: Transform) -> Self {
        let collider = Collider::rectangle(Wheel::LENGTH, Wheel::WIDTH);
        Self {
            wheel: Wheel,
            rigid_body: RigidBody::Dynamic,
            collider,
            spatial: SpatialBundle::from_transform(transform),
            layer: CollisionLayers {
                memberships: LayerMask(1 << Wheel::COLLISION_LAYER),
                filters: LayerMask(1 << Wheel::COLLISION_LAYER),
            },
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
}

impl FrontWheelJointBundle {
    pub(crate) fn new(axle: Entity, wheel: Entity, offset: Vec2) -> Self {
        FrontWheelJointBundle {
            wheel_joint: WheelJoint,
            joint: RevoluteJoint::new(axle, wheel)
                .with_local_anchor_1(offset)
                .with_angle_limits(
                    -Car::MAX_STEERING_DEG.to_radians(),
                    Car::MAX_STEERING_DEG.to_radians(),
                ),
        }
    }
}

#[derive(Bundle)]
pub struct BackWheelJointBundle {
    wheel_joint: WheelJoint,
    joint: FixedJoint,
}

impl BackWheelJointBundle {
    pub(crate) fn new(axle: Entity, wheel: Entity, offset: Vec2) -> Self {
        BackWheelJointBundle {
            wheel_joint: WheelJoint,
            joint: FixedJoint::new(axle, wheel).with_local_anchor_1(offset),
        }
    }
}
