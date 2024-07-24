use avian2d::prelude::{
    AngularVelocity, Collider, ExternalAngularImpulse, ExternalForce, ExternalImpulse, Inertia,
    LinearVelocity, Mass, RigidBody,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};

use bevy_reactive_blueprints::{AsChild, BlueprintPlugin, FromBlueprint};
use physics::DrivingPhysics;

mod physics;

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct AccelerateAction;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct BrakeAction;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct TurnAction(pub f32);

impl CarPlugin {
    fn calculate_driving_physics(
        mut commands: Commands,
        mut car_query: Query<
            (Entity, Option<&mut DrivingData>, &Transform, &TurnAction),
            With<Car>,
        >,
    ) {
        for (entity, prev_data, transform, steering) in car_query.iter_mut() {
            let physics = DrivingPhysics::new(*transform, *steering);

            let driving_data = DrivingData::new(physics);

            if let Some(mut prev_data) = prev_data {
                *prev_data = driving_data
            } else {
                commands.entity(entity).insert(driving_data);
            }
        }
    }

    fn apply_driving_physics(
        mut query: Query<(&DrivingData, &mut Transform, &mut ExternalForce), With<Car>>,
    ) {
        for (physics, mut transform, mut force) in query.iter_mut() {
            **force += physics.force;
            transform.look_to(Vec3::new(physics.force.x, 0., physics.force.y), Vec3::Y);
        }
    }
}

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
    pub const WHEEL_BASIS: f32 = 0.5;
    pub const TURNING_ANGLE: f32 = 18.;
}

#[derive(Clone, Debug, Default)]
#[derive(Bundle)]
pub struct CarBundle {
    pub car: Car,
}

#[derive(Clone, Debug, Default)]
#[derive(Bundle)]
pub struct CarPhysicsBundle {
    rigid_body: RigidBody,
    collider: Collider,
    mass: Mass,
    inertia: Inertia,
    linear_velocity: LinearVelocity,
    angular_velocity: AngularVelocity,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
    external_angular_impulse: ExternalAngularImpulse,
}

#[derive(Bundle)]
pub struct CarGraphicsBundle {
    pbr: PbrBundle,
}

impl CarGraphicsBundle {
    fn from_blueprint(blueprint: &CarBlueprint) -> Self {
        Self {
            pbr: PbrBundle::default(),
        }
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
    type Params<'w, 's> = Res<'w, AssetServer>;

    fn from_blueprint(
        blueprint: &CarBlueprint,
        params: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        CarGraphicsBundle::from_blueprint(blueprint)
    }
}
