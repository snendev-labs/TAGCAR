use bevy::prelude::*;
use sickle_ui::prelude::*;

use car::{CarBlueprint, CarBundle};
use laptag::Score;
use scoreboard::{CarName, Scoreboard, ScoreboardUI};
use tagcar::TagcarPlugins;
use track::{LapComplete, Track, TrackInterior};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    app.add_systems(Startup, (spawn_camera, spawn_game));
    app.add_systems(Update, (slowmo_on_lap_completion, update_score));
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
