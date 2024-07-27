use avian2d::prelude::PhysicsDebugPlugin;
#[cfg(feature = "debug")]
use avian2d::{prelude::Gravity, PhysicsPlugins};
use bevy::{app::PluginGroupBuilder, prelude::*};
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_reactive_blueprints::BlueprintsPlugin;

use car::CarPlugin;
use controller::CarControllerPlugin;
use resurfacer::ResurfacerPlugin;
use track::TrackPlugin;

pub struct TagcarPlugins;

impl PluginGroup for TagcarPlugins {
    fn build(self) -> PluginGroupBuilder {
        let builder = PluginGroupBuilder::start::<Self>();
        #[cfg(feature = "debug")]
        let builder = builder
            .add(WorldInspectorPlugin::default())
            .add(PhysicsDebugPlugin::default());
        builder
            .add(PhysicsPlugin)
            .add(BlueprintsPlugin)
            .add(TrackPlugin)
            .add(CarPlugin)
            .add(ResurfacerPlugin)
            .add(CarControllerPlugin)
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        app.insert_resource(Gravity::ZERO);
    }
}
