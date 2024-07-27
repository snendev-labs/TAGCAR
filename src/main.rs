use bevy::prelude::*;

use bot_controller::BotController;
use camera::CameraTracker;
use car::{Car, CarBlueprint};
use controller::Controller;
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
    let bounds_max = Vec2::new(track.half_length() - 300., track.radius() - 200.);
    let (start_line_center, angle) = track.checkpoints().next().unwrap();
    commands.spawn((
        CarBlueprint::new(start_line_center, angle + std::f32::consts::PI),
        Controller::ArrowKeys,
        CameraTracker::rect(-bounds_max, bounds_max),
    ));
    commands.spawn((
        CarBlueprint::new(
            start_line_center
                + Vec2::from_angle(angle + std::f32::consts::FRAC_PI_2) * Car::WIDTH * 2.,
            angle + std::f32::consts::PI,
        ),
        BotController,
    ));
    let interior = TrackInterior::from_track(&track);
    commands.spawn(interior.bundle());
    commands.spawn(track.bundle());
}
