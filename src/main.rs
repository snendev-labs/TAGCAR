use bevy::prelude::*;

use car::{Car, CarBlueprint};
use controller::Controller;
use track::{Track, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    app.add_systems(Startup, (spawn_camera, spawn_game));

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_game(mut commands: Commands) {
    let track = Track::default();
    let (start_line_center, angle) = track.checkpoints().next().unwrap();
    commands.spawn((
        Controller::ArrowKeys,
        CarBlueprint::new(start_line_center, angle + std::f32::consts::PI),
    ));
    commands.spawn((CarBlueprint::new(
        start_line_center + Vec2::from_angle(angle + std::f32::consts::FRAC_PI_2) * Car::WIDTH * 2.,
        angle + std::f32::consts::PI,
    ),));
    let interior = TrackInterior::from_track(&track);
    commands.spawn(interior.bundle());
    commands.spawn(track.bundle());
}
