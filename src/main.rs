use bevy::prelude::*;

use bot_controller::BotControllerBundle;
use camera::CameraTracker;
use car::{Car, CarBlueprint};
use controller::Controller;
use entropy::{GlobalEntropy, RngCore};
use laptag::{BombTagIt, CanBeIt, LapTagIt, Score};
use track::{CheckpointHighlightTracker, Track, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);
    app.add_systems(Startup, spawn_loading_ui);

    #[cfg(feature = "audio")]
    let run_condition = resource_exists::<bg_music::BgMusicAssets>
        .and_then(resource_exists::<audio_fx::AudioFxAssets>)
        .and_then(run_once());
    #[cfg(not(feature = "audio"))]
    let run_condition = run_once();
    app.add_systems(Update, (spawn_game, despawn_ui).run_if(run_condition));
    app.run();
}

fn spawn_game(mut commands: Commands, mut entropy: ResMut<GlobalEntropy>) {
    let track = Track::default();
    let (first_chunk, second_chunk, third_chunk) = {
        let mut chunks = track.chunks();
        let first_chunk = chunks.next().unwrap();
        let second_chunk = chunks.next().unwrap();
        let third_chunk = chunks.next().unwrap();
        (first_chunk, second_chunk, third_chunk)
    };
    let bounds_max = Vec2::new(track.half_length() - 300., track.radius() - 200.);

    const CAR_COUNT: usize = 5;
    let mut cars_to_spawn = (0..CAR_COUNT).collect::<Vec<_>>();
    cars_to_spawn.sort_by_cached_key(|_| entropy.next_u32());
    for (position_index, car_index) in cars_to_spawn.into_iter().enumerate() {
        let start_offset = track.interior_radius() + Car::WIDTH;
        let car_index_offset =
            (position_index as f32 / CAR_COUNT as f32) * (track.thickness() - Car::WIDTH * 2.);
        // the car with scoring tag starts ahead
        // and the car with bomb tag starts behind
        let spawn_chunk = if car_index == 1 {
            third_chunk.clone()
        } else if car_index == 2 {
            first_chunk.clone()
        } else {
            second_chunk.clone()
        };
        let spawn_angle = spawn_chunk.angle() + std::f32::consts::FRAC_PI_2;
        let spawn_position = spawn_chunk.origin()
            + Vec2::from_angle(spawn_chunk.angle()) * (start_offset + car_index_offset);

        let mut builder = commands.spawn((
            CanBeIt,
            Score::default(),
            CarBlueprint::new(spawn_position, spawn_angle),
        ));
        match car_index {
            0 => {
                builder.insert((
                    BotControllerBundle::new(entropy.as_mut()),
                    CameraTracker::rect(-bounds_max, bounds_max),
                    CheckpointHighlightTracker,
                ));
            }
            1 => {
                builder.insert((BotControllerBundle::new(entropy.as_mut()), LapTagIt));
            }
            2 => {
                builder.insert((BotControllerBundle::new(entropy.as_mut()), BombTagIt));
            }
            _ => {
                builder.insert(BotControllerBundle::new(entropy.as_mut()));
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
