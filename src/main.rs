use avian2d::prelude::*;
use bevy::prelude::*;

use track::{Track, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    app.add_plugins(PhysicsDebugPlugin::default());
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
