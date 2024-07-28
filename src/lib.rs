#[cfg(feature = "debug")]
use avian2d::prelude::PhysicsDebugPlugin;
use avian2d::{prelude::Gravity, PhysicsPlugins};
use bevy::{app::PluginGroupBuilder, input::common_conditions::input_just_pressed, prelude::*};
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};
use bevy_reactive_blueprints::BlueprintsPlugin;

pub struct TagcarPlugins;

impl PluginGroup for TagcarPlugins {
    fn build(self) -> PluginGroupBuilder {
        let builder = PluginGroupBuilder::start::<Self>();
        #[cfg(feature = "debug")]
        let builder = builder
            .add(WorldInspectorPlugin::default())
            .add(PhysicsDebugPlugin::default());
        builder
            .add(EntropyPlugin::<WyRand>::default())
            .add(PhysicsPlugin)
            .add(BlueprintsPlugin)
            .add(car::CarPlugin)
            .add(controller::CarControllerPlugin)
            .add(track::TrackPlugin)
            .add(resurfacer::ResurfacerPlugin)
            .add(scoreboard::ScoreboardPlugin)
            .add(SlowmoPlugin)
            .add(bot_controller::BotControllerPlugin)
            .add(camera::GameCameraPlugin)
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        app.insert_resource(Gravity::ZERO);
    }
}

pub struct SlowmoPlugin;

impl Plugin for SlowmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(slowmo::SlowmoPlugin).configure_sets(
            Update,
            slowmo::TriggerSlowmoSystems.run_if(
                on_event::<track::LapComplete>().or_else(input_just_pressed(KeyCode::KeyP)),
            ),
        );
    }
}
