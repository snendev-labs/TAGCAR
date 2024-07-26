use avian2d::prelude::{
    AngularVelocity, Collider, ExternalAngularImpulse, ExternalForce, ExternalImpulse, Inertia,
    LinearVelocity, Mass, RigidBody,
};
use bevy::{
    color::palettes::css::RED,
    ecs::system::StaticSystemParam,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use bevy_reactive_blueprints::{AsChild, BlueprintPlugin, FromBlueprint};
use physics::DrivingPhysics;

mod physics;

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BlueprintPlugin::<CarBlueprint, TotalCarBundle>::default(),
            BlueprintPlugin::<CarBlueprint, CarGraphicsBundle, AsChild>::default(),
        ))
        .add_systems(
            Update,
            (
                Self::calculate_driving_physics,
                Self::apply_driving_physics,
                Self::clear_action_components,
            )
                .chain()
                .in_set(DrivingSystems),
        );
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub enum AccelerateAction {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct TurnAction(pub f32);

impl CarPlugin {
    fn calculate_driving_physics(
        mut commands: Commands,
        mut car_query: Query<
            (
                Entity,
                Option<&mut DrivingData>,
                &Transform,
                Option<&TurnAction>,
                &AccelerateAction,
            ),
            With<Car>,
        >,
    ) {
        for (entity, prev_data, transform, steering, accelerate) in &mut car_query {
            let steering = steering.unwrap_or(&TurnAction(0.));
            println!("Acceleration registered");
            let physics = DrivingPhysics::new(*transform, *steering, *accelerate);

            let driving_data = DrivingData::new(physics);

            if let Some(mut prev_data) = prev_data {
                println!("Driving data changed");
                *prev_data = driving_data
            } else {
                println!("Driving data added");
                commands.entity(entity).insert(driving_data);
            }
        }
    }

    fn apply_driving_physics(
        mut query: Query<
            (
                &DrivingData,
                &mut Transform,
                &mut ExternalForce,
                &LinearVelocity,
            ),
            With<Car>,
        >,
    ) {
        for (physics, mut transform, mut force, velocity) in query.iter_mut() {
            println!("{:?}", physics.force);
            println!("Applying physics");
            **force += physics.force;
            println!("{:?}", force);
            println!("{:?}", velocity);
            transform.look_to(Vec3::new(physics.force.x, physics.force.y, 0.), Vec3::Y);
        }
    }

    fn clear_action_components(mut commands: Commands, car_query: Query<Entity, With<Car>>) {
        for car_entity in &car_query {
            println!("Driving components cleared");
            commands
                .entity(car_entity)
                .remove::<TurnAction>()
                .remove::<AccelerateAction>();
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct DrivingSystems;

#[derive(Clone, Debug)]
#[derive(Component, Reflect)]
pub struct DrivingData {
    pub state: DrivingPhysics,
    pub force: Vec2,
}

impl DrivingData {
    pub fn new(state: DrivingPhysics) -> Self {
        let force = state.calculate_force();
        DrivingData { state, force }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Car;

impl Car {
    pub const ENGINE_POWER: f32 = 100.;
    pub const REVERSE_POWER: f32 = 50.;
    pub const WHEEL_BASIS: f32 = 0.5;
    pub const TURNING_ANGLE: f32 = 18.;
}

#[derive(Clone, Debug, Default)]
#[derive(Bundle)]
pub struct CarBundle {
    pub car: Car,
}

#[derive(Clone, Debug)]
#[derive(Bundle)]
pub struct CarPhysicsBundle {
    rigid_body: RigidBody,
    collider: Collider,
    mass: Mass,
    inertia: Inertia,
    linear_velocity: LinearVelocity,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
}

impl Default for CarPhysicsBundle {
    fn default() -> Self {
        CarPhysicsBundle {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(50.),
            mass: Mass(10.),
            inertia: Default::default(),
            linear_velocity: Default::default(),
            external_force: Default::default(),
            external_impulse: Default::default(),
        }
    }
}

#[derive(Bundle)]
pub struct CarGraphicsBundle {
    pub shape: MaterialMesh2dBundle<ColorMaterial>,
}

impl CarGraphicsBundle {
    pub fn new(shape: MaterialMesh2dBundle<ColorMaterial>) -> Self {
        CarGraphicsBundle { shape }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Reflect)]
pub struct CarBlueprint {
    pub origin: Vec3,
}

pub type TotalCarBundle = (CarBundle, CarPhysicsBundle);

impl FromBlueprint<CarBlueprint> for TotalCarBundle {
    type Params<'w, 's> = ();

    fn from_blueprint(
        _blueprint: &CarBlueprint,
        _: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        (CarBundle::default(), CarPhysicsBundle::default())
    }
}

impl FromBlueprint<CarBlueprint> for CarGraphicsBundle {
    type Params<'w, 's> = (ResMut<'w, Assets<Mesh>>, ResMut<'w, Assets<ColorMaterial>>);

    fn from_blueprint(
        blueprint: &CarBlueprint,
        params: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        CarGraphicsBundle {
            shape: MaterialMesh2dBundle {
                mesh: Mesh2dHandle(params.0.add(Rectangle::new(1.0, 1.0))),
                material: params.1.add(Color::from(RED)),
                transform: Transform::from_translation(blueprint.origin),
                ..default()
            },
        }
    }
}
