use avian2d::PhysicsPlugins;
use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_reactive_blueprints::BlueprintsPlugin;

use track::TrackPlugin;

pub struct TagcarPlugins;

impl PluginGroup for TagcarPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add_group(PhysicsPlugins::default())
            .add(BlueprintsPlugin)
            .add(TrackPlugin)
    }
}
