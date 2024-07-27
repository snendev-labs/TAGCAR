use bevy::prelude::*;

use car::CarBlueprint;
use laptag::{BombTagIt, ScoreTagIt};
use track::{Track, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    app.add_systems(Startup, (spawn_camera, spawn_game));
    app.add_systems(Update, force_particle_command);
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_game(mut commands: Commands) {
    let track = Track::default();
    let (start_line_center, angle) = track.checkpoints().next().unwrap();
    commands.spawn(CarBlueprint::new(
        start_line_center,
        angle + std::f32::consts::PI,
    ));
    let interior = TrackInterior::from_track(&track);
    commands.spawn(interior.bundle());
    commands.spawn(track.bundle());
}

fn force_particle_command(mut commands: Commands, inputs: Res<ButtonInput<KeyCode>>) {
    if inputs.just_pressed(KeyCode::KeyQ) {
        commands.add(ScoreTagIt::spawn_effects(Vec2::splat(10.)));
    }
    if inputs.just_pressed(KeyCode::KeyW) {
        commands.add(BombTagIt::spawn_effects(Vec2::splat(10.)));
    }
}
