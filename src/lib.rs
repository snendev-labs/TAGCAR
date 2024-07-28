#[cfg(feature = "debug")]
use avian2d::prelude::PhysicsDebugPlugin;
use avian2d::{prelude::Gravity, PhysicsPlugins};
use bevy::{app::PluginGroupBuilder, input::common_conditions::input_just_pressed, prelude::*};
#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_kira_audio::AudioPlugin;
use bevy_reactive_blueprints::BlueprintsPlugin;

use camera::GameCamera;
use entropy::EntropyPlugin;
use laptag::TagEvent;
use track::LapComplete;

pub struct TagcarPlugins;

impl PluginGroup for TagcarPlugins {
    fn build(self) -> PluginGroupBuilder {
        let builder = PluginGroupBuilder::start::<Self>();
        #[cfg(feature = "debug")]
        let builder = builder
            .add(WorldInspectorPlugin::default())
            .add(PhysicsDebugPlugin::default());
        builder
            .add_group(PhysicsPlugins::default())
            .add(AudioPlugin)
            .add(EntropyPlugin)
            .add(BlueprintsPlugin)
            .add(car::CarPlugin)
            .add(controller::CarControllerPlugin)
            .add(track::TrackPlugin)
            .add_group(laptag::LapTagPlugins)
            .add(resurfacer::ResurfacerPlugin)
            .add(scoreboard::ScoreboardPlugin)
            .add(slowmo::SlowmoPlugin)
            .add(bot_controller::BotControllerPlugin)
            .add(camera::GameCameraPlugin)
            .add(bg_music::BgMusicPlugin)
            .add(audio_fx::AudioFxPlugin)
            .add(IntegrationPlugin)
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
        app.configure_sets(
            Update,
            audio_fx::AudioFxSystems::CrashFx.run_if(event_occurs_on_camera::<TagEvent>),
        );
        app.configure_sets(
            Update,
            audio_fx::AudioFxSystems::ScoreFx.run_if(event_occurs_on_camera::<LapComplete>),
        );
        app.configure_sets(
            Update,
            slowmo::TriggerSlowmoSystems.run_if(
                event_occurs_on_camera::<LapComplete>
                    .or_else(event_occurs_on_camera::<TagEvent>)
                    .or_else(input_just_pressed(KeyCode::KeyP)),
            ),
        );
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
