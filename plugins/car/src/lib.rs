use avian2d::prelude::{
    AngularVelocity, ExternalAngularImpulse, ExternalImpulse, ExternalTorque, Gravity,
    LinearVelocity, Rotation,
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
                Self::reset_overspinning_objects,
                Self::apply_steering,
                Self::apply_acceleration,
                Self::apply_wheel_friction,
                Self::apply_car_drag,
                Self::clear_action_components,
                Self::despawn_car_parts,
                Self::spawn_car_parts,
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
    fn reset_overspinning_objects(
        mut objects: Query<(
            &mut AngularVelocity,
            &mut ExternalAngularImpulse,
            &mut ExternalTorque,
            Option<&CarParts>,
        )>,
    ) {
        let mut second_pass_entities = vec![];
        for (mut velocity, mut impulse, mut torque, parts) in &mut objects {
            if velocity.0 > 100. {
                velocity.0 = 0.;
                impulse.set_impulse(0.);
                torque.set_torque(0.);
                if let Some(parts) = parts {
                    second_pass_entities.extend([
                        parts.wheel_front_left,
                        parts.wheel_front_right,
                        parts.wheel_back_left,
                        parts.wheel_back_right,
                    ]);
                }
            }
        }
        for entity in second_pass_entities {
            let Ok((mut velocity, mut impulse, mut torque, _)) = objects.get_mut(entity) else {
                continue;
            };
            velocity.0 = 0.;
            impulse.set_impulse(0.);
            torque.set_torque(0.);
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
                .flatten()
            {
                if let Some(SteerAction(steer_angle)) = steering {
                    **impulse += *steer_angle * 100.;
                } else {
                    let wheel_rotation = car_rotation.angle_between(*wheel_rotation);
                    if wheel_rotation > 1_f32.to_radians() {
                        **impulse -= 100. * wheel_rotation * std::f32::consts::FRAC_1_PI;
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
        for (mut impulse, velocity, rotation) in &mut wheels {
            if velocity.length() <= f32::EPSILON {
                continue;
            }
            // higher friction when close to stopped
            let forward = Vec2::from_angle(rotation.as_radians());
            // friction against the ground is proportional to the force of gravity exerted by the wheel
            // each wheel should share about a quarter of the car's weight
            let force_against_ground =
                (Wheel::MASS.0 + Car::MASS.0 / 4.) * Gravity::default().0.length();

            let main_axis_friction = if velocity.dot(forward).is_sign_positive() {
                // main-axis friction with velocity facing forward can be calculated using the projection of the normalized
                // velocity vector onto the forward vector
                -0.8 * velocity.normalize().project_onto(forward) * force_against_ground
            } else if velocity.dot(forward).is_sign_positive() {
                // main-axis friction with velocity opposite forward is lower, since the car is slipping
                -0.3 * velocity.normalize().project_onto(forward) * force_against_ground
            } else {
                Vec2::ZERO
            };
            // in the cross-axis direction, friction is much higher
            let cross_axis_friction =
                -4. * velocity.normalize().reject_from(forward) * force_against_ground;
            **impulse += main_axis_friction + cross_axis_friction;
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

    fn despawn_car_parts(
        mut commands: Commands,
        mut removed_cars: RemovedComponents<Car>,
        parts: Query<(Entity, &PartOfCar)>,
    ) {
        let removed_cars = removed_cars.read().collect::<Vec<_>>();
        if removed_cars.is_empty() {
            return;
        }
        for (entity, part_of_car) in &parts {
            if removed_cars.contains(part_of_car) {
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn spawn_car_parts(
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
                    WheelBundle::new(car, add_offset(transform, front_right_offset)),
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
                    WheelBundle::new(car, add_offset(transform, front_left_offset)),
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
                    WheelBundle::new(car, add_offset(transform, back_right_offset)),
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
                    WheelBundle::new(car, add_offset(transform, back_left_offset)),
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
