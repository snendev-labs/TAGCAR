use avian2d::prelude::*;
use bevy::prelude::*;

pub struct DrivingPhysics {
    pub transform: Transform,
    pub front_wheel_angle: f32,
    pub linear_velocity: Vec2,
    pub delta_time: f32,
}

impl DrivingPhysics {
    const ENGINE_POWER: f32 = 100.;
    const WHEEL_BASIS: f32 = 0.5;

    pub fn calculate_velocity(&self) -> Vec3 {
        let mut forward = self.transform.forward().as_vec3();
        let acceleration = forward * Self::ENGINE_POWER;
        let mut velocity = acceleration * self.delta_time;

        let origin = self.transform.translation;
        let mut front_wheel = origin + forward * Self::WHEEL_BASIS;
        let mut back_wheel = origin - forward * Self::WHEEL_BASIS;
        back_wheel += velocity * self.delta_time;
        front_wheel +=
            Quat::from_rotation_z(self.front_wheel_angle).mul_vec3(forward) * self.delta_time;
        forward = (front_wheel - back_wheel).normalize();
        velocity = forward * velocity.length();
        velocity
    }
}
