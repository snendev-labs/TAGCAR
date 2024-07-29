#[cfg(feature = "debug-all")]
use avian2d::prelude::PhysicsDebugPlugin;
use avian2d::{prelude::Gravity, PhysicsPlugins};
use bevy::{app::PluginGroupBuilder, prelude::*};
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_reactive_blueprints::BlueprintsPlugin;

use bot_controller::BotControllerBundle;
use camera::{CameraTracker, GameCamera};
use car::{Car, CarBlueprint};
use controller::Controller;
use entropy::{EntropyPlugin, GlobalEntropy, RngCore};
use laptag::{BombTagIt, CanBeIt, LapTagIt, Score, TagEvent};
use track::{CheckpointHighlightTracker, LapComplete, Track, TrackChunk};

mod game_loop;
pub use game_loop::Player;

pub struct TagcarPlugins;

impl PluginGroup for TagcarPlugins {
    fn build(self) -> PluginGroupBuilder {
        let builder = PluginGroupBuilder::start::<Self>();
        #[cfg(feature = "debug")]
        let builder = builder.add(WorldInspectorPlugin::default());
        #[cfg(feature = "debug-all")]
        let builder = builder.add(PhysicsDebugPlugin::default());
        let builder = builder
            .add_group(PhysicsPlugins::default())
            .add(EntropyPlugin)
            .add(BlueprintsPlugin)
            .add(car::CarPlugin)
            .add(controller::CarControllerPlugin)
            .add(track::TrackPlugin)
            .add_group(laptag::LapTagPlugins)
            .add(resurfacer::ResurfacerPlugin)
            .add(scoreboard::ScoreboardPlugin)
            // .add(slowmo::SlowmoPlugin)
            .add(bot_controller::BotControllerPlugin)
            .add(camera::GameCameraPlugin)
            .add(IntegrationPlugin)
            .add(game_loop::GameLoopPlugin);
        #[cfg(feature = "audio")]
        let builder = builder
            .add(bevy_kira_audio::AudioPlugin)
            .add(bg_music::BgMusicPlugin)
            .add(audio_fx::AudioFxPlugin);
        builder
    }
}

pub struct IntegrationPlugin;

impl Plugin for IntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity::ZERO);
        app.configure_sets(
            Update,
            camera::GameCameraSystems::Shake.run_if(event_occurs_on_camera::<TagEvent>),
        );
        #[cfg(feature = "audio")]
        app.configure_sets(
            Update,
            audio_fx::AudioFxSystems::CrashFx.run_if(event_occurs_on_camera::<TagEvent>),
        );
        #[cfg(feature = "audio")]
        app.configure_sets(
            Update,
            audio_fx::AudioFxSystems::ScoreFx.run_if(event_occurs_on_camera::<LapComplete>),
        );
        // TODO: Slowmo just makes the game feel laggy. zoom in or something?
        // app.configure_sets(
        //     Update,
        //     slowmo::TriggerSlowmoSystems.run_if(
        //         event_occurs_on_camera::<LapComplete>
        //             .or_else(event_occurs_on_camera::<TagEvent>)
        //             .or_else(input_just_pressed(KeyCode::KeyP)),
        //     ),
        // );
    }
}

trait GetEntities {
    fn entities(&self) -> impl Iterator<Item = Entity> + '_;
}

impl GetEntities for LapComplete {
    fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        std::iter::once(self.racer)
    }
}

impl GetEntities for TagEvent {
    fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        [self.prev_it, self.next_it].into_iter()
    }
}

fn event_occurs_on_camera<E: GetEntities + Event>(
    mut tag_events: EventReader<E>,
    positions: Query<&Transform>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) -> bool {
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return false;
    };
    for entity in tag_events.read().flat_map(|event| event.entities()) {
        let Ok(transform) = positions.get(entity) else {
            continue;
        };
        let Some(viewport_position) =
            camera.world_to_viewport(camera_transform, transform.translation)
        else {
            continue;
        };
        let Some(view_rect) = camera.logical_viewport_rect() else {
            continue;
        };
        if view_rect.contains(viewport_position) {
            return true;
        }
    }
    return false;
}

pub fn spawn_cars(commands: &mut Commands, track: &Track, entropy: &mut GlobalEntropy) {
    let chunks = track.chunks().collect::<Vec<_>>();
    let bounds_max = Vec2::new(track.half_length() - 300., track.radius() - 200.);

    // from back to front, we spawn:
    // a bomb holder in the center of the checkpoint
    // a COLxROW grid of not-IT players in the next ROW checkpoints
    // a flag holder in the center of the ROW+1 checkpoint

    const ROW_COUNT: usize = 2;
    const COL_COUNT: usize = 4;
    const GRID_COUNT: usize = ROW_COUNT * COL_COUNT;

    // spawn bomb holder
    commands.spawn((
        BotControllerBundle::new(entropy),
        BombTagIt,
        car_from_track(
            &track,
            chunks.get(0).expect("Cars to spawn on known checkpoints"),
            0.5,
            false,
        ),
    ));

    // spawn the grid, including the player
    let random_grid_index = entropy.next_u32() as f32 / u32::MAX as f32 * GRID_COUNT as f32;
    let cars = (0..ROW_COUNT)
        .flat_map(|row_index| (0..COL_COUNT).map(move |col_index| (col_index, row_index)))
        .map(|(col_index, row_index)| {
            car_from_track(
                &track,
                chunks
                    .get(row_index * 2 + 2 + col_index % 2)
                    .expect("Cars to spawn on known checkpoints"),
                col_index as f32 / COL_COUNT as f32,
                col_index + row_index * COL_COUNT == random_grid_index as usize,
            )
        })
        .collect::<Vec<_>>();
    for (index, car) in cars.into_iter().enumerate() {
        if index == random_grid_index as usize {
            // this one is the player
            commands.spawn((
                car,
                Player,
                Controller::ArrowKeys,
                CameraTracker::rect(-bounds_max, bounds_max),
                CheckpointHighlightTracker,
            ));
        } else {
            commands.spawn((car, BotControllerBundle::new(entropy)));
        }
    }

    // spawn flag holder
    commands.spawn((
        BotControllerBundle::new(entropy),
        LapTagIt,
        car_from_track(
            &track,
            chunks
                .get(ROW_COUNT * 2 + 3)
                .expect("Cars to spawn on known checkpoints"),
            0.5,
            false,
        ),
    ));
}

fn car_from_track(
    track: &Track,
    chunk: &TrackChunk,
    offset_along_line: f32,
    is_player: bool,
) -> impl Bundle {
    let start_offset: f32 = track.interior_radius() + Car::WIDTH;
    let car_index_offset = offset_along_line * (track.thickness() - Car::WIDTH * 2.);
    // the car with scoring tag starts ahead
    // and the car with bomb tag starts behind
    let spawn_angle = chunk.angle() + std::f32::consts::FRAC_PI_2;
    let spawn_position =
        chunk.origin() + Vec2::from_angle(chunk.angle()) * (start_offset + car_index_offset);
    (
        CarBlueprint::new(spawn_position, spawn_angle, is_player),
        Score::default(),
        CanBeIt,
    )
}
