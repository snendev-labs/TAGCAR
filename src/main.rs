use bevy::prelude::*;

use camera::CameraTracker;
use car::CarBlueprint;
use track::{Checkpoint, Track, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    app.add_systems(Startup, spawn_game);

    app.run();
}

fn spawn_game(mut commands: Commands) {
    let track = Track::default();
    let chunk = track.chunks().next().unwrap();
    let bounds_max = Vec2::new(track.half_length() - 300., track.radius() - 200.);
    let chunk = Checkpoint::from_chunk(&track, chunk, 0);
    commands.spawn((
        CarBlueprint::new(chunk.position, chunk.angle + std::f32::consts::FRAC_PI_2),
        CameraTracker::rect(-bounds_max, bounds_max),
    ));
    let interior = TrackInterior::from_track(&track);
    commands.spawn(interior.bundle());
    commands.spawn(track.bundle());
}
