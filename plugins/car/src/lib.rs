use avian2d::prelude::{
    AngularVelocity, Collider, ExternalAngularImpulse, ExternalForce, ExternalImpulse, Inertia,
    LinearVelocity, Mass, RigidBody,
};
use bevy::{ecs::system::{StaticSystemParam, SystemParam}, prelude::*};

use bevy_reactive_blueprints::{AsChild, BlueprintPlugin, FromBlueprint};

mod physics;

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlueprintPlugin::<CarBlueprint, TotalCarBundle>::default())
        .add_plugins(BlueprintPlugin::<CarBlueprint, CarGraphicsBundle, AsChild>::default());
    }
}

impl CarPlugin {
    
}

#[derive(Clone, Copy, Debug, Default, Component)]
pub struct Car;

#[derive(Clone, Copy, Debug, Default, Bundle)]
pub struct CarBundle {
    pub car: Car,
}

#[derive(Clone, Debug, Default, Bundle)]
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



#[derive(Clone, Debug, Bundle)]
pub struct CarGraphicsBundle {}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Reflect)]
pub struct CarBlueprint {
    pub origin: Vec3,
}

pub type TotalCarBundle = (CarBundle, CarPhysicsBundle);

impl FromBlueprint<CarBlueprint> for TotalCarBundle {
    type Params<'w, 's> = SystemParam<'w>;

    fn from_blueprint(blueprint: &CarBlueprint, params: &mut StaticSystemParam<Self::Params<'_, '_>>) -> Self {
        (
            CarBundle::default(),
            CarPhysicsBundle::default(),
        )
    }
}

impl FromBlueprint<CarBlueprint> for CarGraphicsBundle {
    type Params<'w, 's> = ;

    fn from_blueprint(blueprint: &CarBlueprint, params: &mut StaticSystemParam<Self::Params<'_, '_>>) -> Self {
        
    }
}
