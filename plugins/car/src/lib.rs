use avian2d::prelude::{
    ExternalAngularImpulse, ExternalImpulse, LinearVelocity, PhysicsLayer, Rotation,
};
use bevy::prelude::*;

use bevy_reactive_blueprints::BlueprintPlugin;

mod car;
pub use car::*;
mod wheel;
pub use wheel::*;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use graphics::*;

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "graphics")]
        app.add_plugins(CarGraphicsPlugin);
        app.add_plugins(BlueprintPlugin::<CarBlueprint, TotalCarBundle>::default());
        app.add_systems(
            Update,
            (
                Self::apply_steering,
                Self::apply_acceleration,
                Self::apply_wheel_friction,
                Self::apply_car_drag,
                Self::clear_action_components,
                Self::spawn_car_wheels,
            )
                .chain()
                .in_set(DrivingSystems),
        );
        app.register_type::<AccelerateAction>()
            .register_type::<SteerAction>()
            .register_type::<Car>()
            .register_type::<CarParts>()
            .register_type::<Wheel>()
            .register_type::<FrontWheel>()
            .register_type::<BackWheel>()
            .register_type::<WheelJoint>();
    }
}

impl CarPlugin {
    fn spawn_car_wheels(
        mut commands: Commands,
        new_cars: Query<(Entity, &Transform), (With<Car>, Without<CarParts>)>,
    ) {
        for (car, transform) in &new_cars {
            let front_right_offset = Wheel::OFFSET;
            let front_left_offset = Wheel::OFFSET * Vec2::new(1., -1.);
            let back_right_offset = Wheel::OFFSET * Vec2::new(-1., 1.);
            let back_left_offset = Wheel::OFFSET * Vec2::new(-1., -1.);

            fn add_offset(transform: &Transform, offset: Vec2) -> Transform {
                let offset = transform.rotation * Vec3::new(offset.x, offset.y, 0.);
                transform
                    .with_translation(transform.translation + Vec3::new(offset.x, offset.y, 0.))
            }

            // front right wheel
            let wheel_front_right = commands
                .spawn((
                    Name::new("Wheel (F,R)"),
                    FrontWheel,
                    WheelBundle::new(add_offset(transform, front_right_offset)),
                ))
                .id();
            let joint_front_right = commands
                .spawn((
                    Name::new("Wheel Joint (F,R)"),
                    FrontWheel,
                    FrontWheelJointBundle::new(car, wheel_front_right, front_right_offset),
                ))
                .id();
            // front left wheel
            let wheel_front_left = commands
                .spawn((
                    Name::new("Wheel (F,L)"),
                    FrontWheel,
                    WheelBundle::new(add_offset(transform, front_left_offset)),
                ))
                .id();
            let joint_front_left = commands
                .spawn((
                    Name::new("Wheel Joint (F,L)"),
                    FrontWheel,
                    FrontWheelJointBundle::new(car, wheel_front_left, front_left_offset),
                ))
                .id();
            // back right wheel
            let wheel_back_right = commands
                .spawn((
                    Name::new("Wheel (B,R)"),
                    BackWheel,
                    WheelBundle::new(add_offset(transform, back_right_offset)),
                ))
                .id();
            let joint_back_right = commands
                .spawn((
                    Name::new("Wheel Joint (B,R)"),
                    BackWheel,
                    BackWheelJointBundle::new(car, wheel_back_right, back_right_offset),
                ))
                .id();
            let wheel_back_left = commands
                .spawn((
                    Name::new("Wheel (B,L)"),
                    BackWheel,
                    WheelBundle::new(add_offset(transform, back_left_offset)),
                ))
                .id();
            let joint_back_left = commands
                .spawn((
                    Name::new("Wheel Joint (B,L)"),
                    BackWheel,
                    BackWheelJointBundle::new(car, wheel_back_left, back_left_offset),
                ))
                .id();
            commands.entity(car).insert(CarParts {
                wheel_front_right,
                joint_front_right,
                wheel_front_left,
                joint_front_left,
                wheel_back_right,
                joint_back_right,
                wheel_back_left,
                joint_back_left,
            });
        }
    }

    fn apply_steering(
        cars: Query<(&Rotation, &CarParts, Option<&SteerAction>), With<Car>>,
        mut front_wheels: Query<(&Rotation, &mut ExternalAngularImpulse), With<FrontWheel>>,
    ) {
        for (car_rotation, parts, steering) in &cars {
            for (wheel_rotation, mut impulse) in front_wheels
                .get_many_mut([parts.wheel_front_left, parts.wheel_front_right])
                .into_iter()
                .flat_map(|data| data)
            {
                if let Some(SteerAction(steer_angle)) = steering {
                    **impulse += *steer_angle * 40.;
                } else {
                    let wheel_rotation = car_rotation.angle_between(*wheel_rotation);
                    if wheel_rotation > 1_f32.to_radians() {
                        **impulse -= 40. * wheel_rotation * std::f32::consts::FRAC_1_PI;
                    }
                }
            }
        }
    }

    fn apply_acceleration(
        cars: Query<(&CarParts, &AccelerateAction), With<Car>>,
        mut wheels: Query<(&Rotation, &mut ExternalImpulse), With<Wheel>>,
    ) {
        for (car_wheels, acceleration) in &cars {
            let CarParts {
                wheel_front_left,
                wheel_front_right,
                wheel_back_left,
                wheel_back_right,
                ..
            } = car_wheels;
            for (rotation, mut impulse) in wheels
                .get_many_mut([
                    *wheel_front_left,
                    *wheel_front_right,
                    *wheel_back_left,
                    *wheel_back_right,
                ])
                .expect("Car wheels to be valid entities")
            {
                let forward = Vec2::from_angle(rotation.as_radians());
                let power = match acceleration {
                    AccelerateAction::Forward => Car::ENGINE_POWER,
                    AccelerateAction::Backward => Car::REVERSE_POWER,
                };
                **impulse += forward * power;
            }
        }
    }

    fn apply_wheel_friction(
        mut wheels: Query<(&mut ExternalImpulse, &LinearVelocity, &Rotation), With<Wheel>>,
    ) {
        const FRICTION: f32 = -0.9;
        for (mut impulse, velocity, rotation) in &mut wheels {
            if velocity.length() <= std::f32::EPSILON {
                continue;
            }
            // higher friction when close to stopped
            let forward = Vec2::from_angle(rotation.as_radians());
            let friction = **velocity * FRICTION;
            let slip_angle: f32 = velocity.angle_between(forward);
            let friction = if slip_angle.to_degrees() < 30. {
                friction
            } else {
                friction + friction.reject_from(forward) * (1. + slip_angle.sin() * 9.)
            };
            **impulse += friction;
        }
    }

    fn apply_car_drag(mut cars: Query<(&mut ExternalImpulse, &LinearVelocity), With<Car>>) {
        const DRAG: f32 = -0.0015;
        for (mut impulse, velocity) in &mut cars {
            if velocity.length_squared() < 25. {
                continue;
            }
            **impulse += **velocity * velocity.length() * DRAG;
        }
    }

    fn clear_action_components(mut commands: Commands, car_query: Query<Entity, With<Car>>) {
        for car_entity in &car_query {
            commands
                .entity(car_entity)
                .remove::<SteerAction>()
                .remove::<AccelerateAction>();
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct DrivingSystems;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub enum AccelerateAction {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Deref, Reflect)]
pub struct SteerAction(pub f32);

#[derive(Clone, Copy, Debug)]
#[derive(PhysicsLayer)]
pub enum CarCollisionLayer {
    Car,
    Wheel,
}
