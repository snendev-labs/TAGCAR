use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{Car, TurnAction};

pub struct DrivingPhysics {
    pub transform: Transform,
    pub front_wheel_angle: TurnAction,
}

impl DrivingPhysics {
    pub fn new(transform: Transform, front_wheel_angle: TurnAction) -> Self {
        DrivingPhysics {
            transform,
            front_wheel_angle,
        }
    }
    pub fn calculate_force(&self) -> Vec2 {
        let forward_vec3 = self.transform.forward().as_vec3();
        let forward = Vec2::new(forward_vec3.x, forward_vec3.z);
        let rotated_forward_vec3 =
            Quat::from_rotation_y(self.front_wheel_angle.0.to_radians()).mul_vec3(forward_vec3);
        let rotated_forward = Vec2::new(rotated_forward_vec3.x, rotated_forward_vec3.z);

        // let origin_vec3 = self.transform.translation;
        // let origin = Vec2::new(origin_vec3.x, origin_vec3.z);
        // let front_wheel = origin + forward * Self::WHEEL_BASIS;
        // let back_wheel = origin - forward * Self::WHEEL_BASIS;
        // let rotated_front_wheel_vec3 =
        //     Quat::from_rotation_y(self.front_wheel_angle.0.to_radians()).mul_vec3(forward_vec3);
        // let rotated_front_wheel = Vec2::new(rotated_front_wheel_vec3.x, rotated_front_wheel_vec3.z);

        let force_direction = (forward + rotated_forward).normalize();
        force_direction * Car::ENGINE_POWER
    }
}
