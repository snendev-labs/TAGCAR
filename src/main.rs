use std::time::Duration;

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;

use track::{LapComplete, Track, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    #[cfg(feature = "debug")]
    app.add_plugins(avian2d::prelude::PhysicsDebugPlugin::default());
    app.add_systems(Startup, (spawn_camera, spawn_track));
    app.add_systems(Update, slowmo_on_lap_completion);
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

fn slowmo_on_lap_completion(
    mut completed_laps: EventReader<LapComplete>,
    real_time: Res<Time<Real>>,
    mut physics_time: ResMut<Time<Physics>>,
    mut slowmo_timer: Local<Option<Duration>>,
) {
    let laps = completed_laps.read().count();
    if laps > 0 {
        physics_time.set_relative_speed(0.25);
        *slowmo_timer = Some(Duration::from_secs(2));
    }
    if let Some(timer) = slowmo_timer.as_mut() {
        *timer = timer.saturating_sub(real_time.delta());
        let relative_speed =
            simple_easing::expo_in((Duration::from_secs(2) - *timer).as_secs_f32() / 2.0);
        physics_time.set_relative_speed(0.25 + 0.75 * relative_speed);
        if timer.is_zero() {
            *slowmo_timer = None;
        }
    }
}
