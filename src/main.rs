use audio_fx::AudioFxAssets;
use bevy::prelude::*;

use bg_music::BgMusicAssets;
use bot_controller::BotController;
use camera::CameraTracker;
use car::{Car, CarBlueprint};
use controller::Controller;
use entropy::{ForkableRng, GlobalEntropy, RngCore};
use laptag::{BombTagIt, CanBeIt, LapTagIt, Score};
use track::{CheckpointHighlightTracker, Track, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    app.add_systems(Startup, spawn_loading_ui);
    app.add_systems(
        Update,
        (spawn_game, despawn_ui).run_if(
            resource_exists::<BgMusicAssets>
                .and_then(resource_exists::<AudioFxAssets>)
                .and_then(run_once()),
        ),
    );
    app.run();
}

fn spawn_game(mut commands: Commands, mut entropy: ResMut<GlobalEntropy>) {
    let track = Track::default();
    let first_chunk = track.chunks().next().unwrap();
    let bounds_max = Vec2::new(track.half_length() - 300., track.radius() - 200.);

    const CAR_COUNT: usize = 5;
    let mut cars_to_spawn = (0..CAR_COUNT).collect::<Vec<_>>();
    cars_to_spawn.sort_by_cached_key(|_| entropy.next_u32());
    let spawn_angle = first_chunk.angle() + std::f32::consts::FRAC_PI_2;
    for (position_index, car_index) in cars_to_spawn.into_iter().enumerate() {
        let start_offset = track.interior_radius() + Car::WIDTH;
        let car_index_offset =
            (position_index as f32 / CAR_COUNT as f32) * (track.thickness() - Car::WIDTH * 2.);
        let spawn_position = first_chunk.origin()
            + Vec2::from_angle(first_chunk.angle()) * (start_offset + car_index_offset);
        let mut builder = commands.spawn((
            CanBeIt,
            Score::default(),
            CarBlueprint::new(spawn_position, spawn_angle),
        ));
        match car_index {
            0 => {
                builder.insert((
                    Controller::ArrowKeys,
                    CameraTracker::rect(-bounds_max, bounds_max),
                    CheckpointHighlightTracker,
                ));
            }
            1 => {
                builder.insert((BotController, entropy.fork_rng(), LapTagIt));
            }
            2 => {
                builder.insert((BotController, entropy.fork_rng(), BombTagIt));
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

#[derive(Component)]
struct LoadingUI;

fn spawn_loading_ui(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Loading UI"),
            LoadingUI,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|builder| {
            builder.spawn(TextBundle::from_section(
                "Loading...",
                TextStyle {
                    font_size: 128.0,
                    color: Color::srgb(0.02, 0.02, 0.1),
                    ..Default::default()
                },
            ));
        });
}

fn despawn_ui(mut commands: Commands, ui_roots: Query<Entity, With<LoadingUI>>) {
    for entity in &ui_roots {
        commands.entity(entity).despawn_recursive();
    }
}
