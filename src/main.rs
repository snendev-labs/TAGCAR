use bevy::prelude::*;

use bot_controller::BotControllerBundle;
use camera::CameraTracker;
use car::{Car, CarBlueprint};
use controller::Controller;
use entropy::{GlobalEntropy, RngCore};
use laptag::{BombTagIt, CanBeIt, LapTagAssets, LapTagIt, Score};
use track::{CheckpointHighlightTracker, Track, TrackAssets, TrackChunk, TrackInterior};

use tagcar::TagcarPlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);

    app.add_systems(Startup, spawn_loading_ui);

    let run_condition = resource_exists::<TrackAssets>
        .and_then(resource_exists::<LapTagAssets>)
        .and_then(run_once());
    #[cfg(feature = "audio")]
    let run_condition = resource_exists::<bg_music::BgMusicAssets>
        .and_then(resource_exists::<audio_fx::AudioFxAssets>)
        .and_then(run_condition);
    app.add_systems(Update, (spawn_game, despawn_ui).run_if(run_condition));

    app.run();
}

fn spawn_game(mut commands: Commands, mut entropy: ResMut<GlobalEntropy>) {
    let track = Track::default();
    let chunks = track.chunks().collect::<Vec<_>>();
    let bounds_max = Vec2::new(track.half_length() - 300., track.radius() - 200.);

    // from back to front, we spawn:
    // a bomb holder in the center of the checkpoint
    // a COLxROW grid of not-IT players in the next ROW checkpoints
    // a flag holder in the center of the ROW+1 checkpoint

    const ROW_COUNT: usize = 3;
    const COL_COUNT: usize = 4;
    const GRID_COUNT: usize = ROW_COUNT * COL_COUNT;

    // spawn bomb holder
    commands.spawn((
        BotControllerBundle::new(entropy.as_mut()),
        BombTagIt,
        car_from_track(
            &track,
            chunks.get(0).expect("Cars to spawn on known checkpoints"),
            0.5,
        ),
    ));

    // spawn the grid, including the player
    let random_grid_index = entropy.next_u32() as f32 / u32::MAX as f32 * GRID_COUNT as f32;
    let cars = {
        let mut cars = (0..ROW_COUNT)
            .flat_map(|row_index| {
                (0..COL_COUNT).map(move |col_index| (col_index, row_index * 2 + 2))
            })
            .map(|(col_index, row_index)| {
                car_from_track(
                    &track,
                    chunks
                        .get(row_index + col_index % 2)
                        .expect("Cars to spawn on known checkpoints"),
                    col_index as f32 / COL_COUNT as f32,
                )
            })
            .collect::<Vec<_>>();
        cars.sort_by_cached_key(|_| entropy.next_u32());
        cars
    };
    for (index, car) in cars.into_iter().enumerate() {
        if index == random_grid_index as usize {
            // this one is the player
            commands.spawn((
                car,
                Controller::WASDKeys,
                CameraTracker::rect(-bounds_max, bounds_max),
                CheckpointHighlightTracker,
            ));
        } else {
            commands.spawn((car, BotControllerBundle::new(entropy.as_mut())));
        }
    }

    // spawn flag holder
    commands.spawn((
        BotControllerBundle::new(entropy.as_mut()),
        LapTagIt,
        car_from_track(
            &track,
            chunks
                .get(ROW_COUNT * 2 + 3)
                .expect("Cars to spawn on known checkpoints"),
            0.5,
        ),
    ));

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

fn car_from_track(track: &Track, chunk: &TrackChunk, offset_along_line: f32) -> impl Bundle {
    let start_offset: f32 = track.interior_radius() + Car::WIDTH;
    let car_index_offset = offset_along_line * (track.thickness() - Car::WIDTH * 2.);
    // the car with scoring tag starts ahead
    // and the car with bomb tag starts behind
    let spawn_angle = chunk.angle() + std::f32::consts::FRAC_PI_2;
    let spawn_position =
        chunk.origin() + Vec2::from_angle(chunk.angle()) * (start_offset + car_index_offset);
    (
        CarBlueprint::new(spawn_position, spawn_angle),
        Score::default(),
        CanBeIt,
    )
}
