use rand_core::RngCore;

use bevy::prelude::*;
use bevy_rand::prelude::{ForkableRng, GlobalEntropy, WyRand};

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

fn spawn_game(mut commands: Commands, mut entropy: ResMut<GlobalEntropy<WyRand>>) {
    let track = Track::default();
    let first_chunk = track.chunks().next().unwrap();
    let bounds_max = Vec2::new(track.half_length() - 300., track.radius() - 200.);

    const CAR_COUNT: usize = 5;
    let mut cars_to_spawn = (0..CAR_COUNT).collect::<Vec<_>>();
    cars_to_spawn.sort_by_cached_key(|_| entropy.next_u32());
    info!("spawning order: {cars_to_spawn:?}");
    let spawn_angle = first_chunk.angle() + std::f32::consts::FRAC_PI_2;
    for (position_index, car_index) in cars_to_spawn.into_iter().enumerate() {
        let start_offset = track.interior_radius() + Car::WIDTH;
        let car_index_offset =
            (position_index as f32 / CAR_COUNT as f32) * (track.thickness() - Car::WIDTH * 2.);
        let spawn_position = first_chunk.origin()
            + Vec2::from_angle(first_chunk.angle()) * (start_offset + car_index_offset);
        let mut builder = commands.spawn(CarBlueprint::new(spawn_position, spawn_angle));
        match car_index {
            0 => {
                builder.insert((
                    Controller::ArrowKeys,
                    CameraTracker::rect(-bounds_max, bounds_max),
                ));
            }
            _ => {
                builder.insert((BotController, entropy.fork_rng()));
            }
        }
    }
    let interior = TrackInterior::from_track(&track);
    commands.spawn(interior.bundle());
    commands.spawn(track.bundle());
}
