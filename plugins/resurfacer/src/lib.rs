use bevy_reactive_blueprints::Blueprint;
use rand_core::RngCore;

use avian2d::prelude::{Collider, RigidBody, Sensor};
use bevy::prelude::*;

use track::{Checkpoint, CheckpointTracker, Track};

mod entropy;
pub use entropy::*;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use graphics::*;

pub struct ResurfacerPlugin;

impl Plugin for ResurfacerPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "graphics")]
        app.add_plugins(GraphicsPlugin);
        app.add_systems(
            Update,
            (Self::spawn_resurfacer, Self::resurface_track)
                .chain()
                .in_set(ResurfacerSystems),
        );
        app.register_type::<Resurfacer>()
            .register_type::<TrackResurfacer>();
    }
}

impl ResurfacerPlugin {
    fn spawn_resurfacer(
        mut commands: Commands,
        tracks: Query<(Entity, &Track), Without<TrackResurfacer>>,
    ) {
        for (track_entity, track) in &tracks {
            let (position, angle) = track
                .checkpoints()
                .next()
                .expect("track to have checkpoints");
            let resurfacer = commands.spawn(Resurfacer::bundle(position, angle)).id();
            commands
                .entity(track_entity)
                .insert(TrackResurfacer(resurfacer));
        }
    }

    fn resurface_track(
        mut commands: Commands,
        mut trackers: Query<
            (&mut CheckpointTracker, &mut Entropy),
            (Changed<CheckpointTracker>, With<Resurfacer>),
        >,
        checkpoints: Query<(&Checkpoint, &Transform)>,
    ) {
        for (mut tracker, mut entropy) in &mut trackers {
            if tracker.len() == 0 {
                continue;
            }
            for entity in tracker.drain() {
                let (checkpoint, transform) = checkpoints
                    .get(entity)
                    .expect("tracker to track valid checkpoint entities");
                while entropy.next_u32() < u32::MAX / 3 {
                    let position_on_checkpoint = entropy.next_u32() as f32 / u32::MAX as f32;
                    let spawn_position = transform.translation.xy()
                        + Vec2::from_angle(checkpoint.angle)
                            * (-0.5 + position_on_checkpoint)
                            * checkpoint.size.y;
                    commands.spawn(Obstacle::IDK.bundle(spawn_position));
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct ResurfacerSystems;

#[derive(Clone, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Resurfacer;

impl Resurfacer {
    const WIDTH: f32 = 30.;
    const Z_INDEX: f32 = 15.;

    fn transform(position: Vec2, angle: f32) -> Transform {
        Transform::from_translation(Vec3::new(position.x, position.y, Self::Z_INDEX))
            .with_rotation(Quat::from_rotation_z(angle))
    }

    pub fn bundle(position: Vec2, angle: f32) -> impl Bundle {
        (
            Resurfacer,
            Blueprint::new(Resurfacer),
            CheckpointTracker::default(),
            Entropy::default(),
            RigidBody::Kinematic,
            Collider::rectangle(Self::WIDTH, Self::WIDTH),
            Sensor,
            SpatialBundle::from_transform(Self::transform(position, angle)),
        )
    }
}

#[derive(Debug)]
#[derive(Component, Deref, Reflect)]
pub struct TrackResurfacer(Entity);

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub enum Obstacle {
    #[default]
    IDK,
}

impl Obstacle {
    const Z_INDEX: f32 = 12.;

    pub fn bundle(self, position: Vec2) -> impl Bundle {
        (
            Blueprint::new(self.clone()),
            self,
            SpatialBundle::from_transform(Transform::from_xyz(
                position.x,
                position.y,
                Self::Z_INDEX,
            )),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use avian2d::{
        prelude::{Physics, PhysicsTime, TimestepMode},
        PhysicsPlugins,
    };
    use bevy::{ecs::system::RunSystemOnce, scene::ScenePlugin};
    use track::{TrackInterior, TrackPlugin};

    fn test_app() -> (App, Entity, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ScenePlugin,
            PhysicsPlugins::default(),
        ));
        app.insert_resource(Time::<Physics>::from_timestep(TimestepMode::FixedOnce {
            delta: Duration::from_secs_f32(1. / 60.),
        }));
        app.add_plugins(TrackPlugin);
        let (e1, e2, e3) = app.world_mut().run_system_once(spawn_track_and_resurfacer);
        (app, e1, e2, e3)
    }

    fn spawn_track_and_resurfacer(mut commands: Commands) -> (Entity, Entity, Entity) {
        let track = Track::default();
        let interior = TrackInterior::from_track(&track);
        let tracker = commands
            .spawn((
                CheckpointTracker::default(),
                Resurfacer,
                RigidBody::Kinematic,
                Collider::rectangle(10., 10.),
                SpatialBundle::from_transform(Transform::from_xyz(track.half_length(), 0., 0.)),
            ))
            .id();
        let interior = commands.spawn(interior.bundle()).id();
        let track = commands.spawn(track.bundle()).id();
        (tracker, track, interior)
    }

    #[test]
    fn test_lap_completion() {
        let (mut app, tracker, track, _) = test_app();

        // let track = app.world_mut().get::<Track>(track).unwrap();
        // for (index, (position, angle)) in track.clone().checkpoints().enumerate() {
        //     let events = app.world_mut().resource::<Events<LapComplete>>();
        //     let mut reader = events.get_reader();
        //     assert!(reader.read(events).find(|lap| ***lap == tracker).is_none());
        //     let reached_checkpoints = app.world_mut().get::<CheckpointTracker>(tracker).unwrap();
        //     assert_eq!(reached_checkpoints.len(), index);
        //     let mut transform = app.world_mut().get_mut::<Transform>(tracker).unwrap();
        //     *transform = Checkpoint::transform(position, angle);
        //     app.update();
        //     app.update();
        // }

        // let events = app.world_mut().resource::<Events<LapComplete>>();
        // let mut reader = events.get_reader();
        // assert!(reader.read(events).find(|lap| ***lap == tracker).is_some());
        // let reached_checkpoints = app.world_mut().get::<CheckpointTracker>(tracker).unwrap();
        // assert_eq!(reached_checkpoints.len(), 0);
    }
}
