use bevy::prelude::*;

use track::{Track, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    #[cfg(feature = "debug")]
    app.add_plugins(avian2d::prelude::PhysicsDebugPlugin::default());
    app.add_systems(Startup, (spawn_camera, spawn_track));
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_track(mut commands: Commands) {
    let track = Track::default();
    let interior = TrackInterior::from_track(&track);
    commands.spawn(interior.bundle());
    commands.spawn(track.bundle());
}
