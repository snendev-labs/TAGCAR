use std::time::Duration;

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;
use sickle_ui::prelude::*;

use car::CarBundle;
use laptag::Score;
use scoreboard::{CarName, Scoreboard, ScoreboardUI};
use track::{LapComplete, Track, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    #[cfg(feature = "debug")]
    app.add_plugins(avian2d::prelude::PhysicsDebugPlugin::default());
    app.add_systems(
        Startup,
        (
            spawn_camera,
            spawn_track,
            spawn_cars,
            spawn_scoreboard,
            insert_timer,
        ),
    );
    app.add_systems(Update, (slowmo_on_lap_completion, update_score));
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

fn spawn_cars(mut commands: Commands) {
    commands
        .spawn(CarBundle::default())
        .insert((CarName("Car 1".to_string()), Score::new(0)));
    commands
        .spawn(CarBundle::default())
        .insert((CarName("Car 2".to_string()), Score::new(0)));
    commands
        .spawn(CarBundle::default())
        .insert((CarName("Car 3".to_string()), Score::new(0)));
}

#[derive(Resource)]
pub struct ScoreTimer {
    timer: Timer,
}

fn insert_timer(mut commands: Commands) {
    commands.insert_resource(ScoreTimer {
        timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
    })
}

fn update_score(
    mut query: Query<(&mut Score, &CarName)>,
    time: Res<Time>,
    mut timer: ResMut<ScoreTimer>,
) {
    timer.timer.tick(time.delta());

    if timer.timer.finished() {
        for (mut score, car_name) in &mut query {
            if car_name.0 == "Car 2" {
                **score += 1;
                println!("Increment car 2 score");
                println!("Car 2 score - {}", score.get());
            }
        }
    }
}

fn spawn_scoreboard(mut commands: Commands) {
    commands.spawn((SpatialBundle::default(), Scoreboard));
}
