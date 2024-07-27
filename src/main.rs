use bevy::prelude::*;

use camera::CameraTracker;
use car::CarBlueprint;
use track::{Track, TrackInterior};

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
    let (start_line_center, angle) = track.checkpoints().next().unwrap();
    let bounds_max = Vec2::new(track.half_length() - 300., track.radius() - 200.);
    commands.spawn((
        CarBlueprint::new(start_line_center, angle + std::f32::consts::PI),
        CameraTracker::rect(-bounds_max, bounds_max),
    ));
    let interior = TrackInterior::from_track(&track);
    commands.spawn(interior.bundle());
    commands.spawn(track.bundle());
}
