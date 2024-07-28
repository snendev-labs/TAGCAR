use bevy_reactive_blueprints::Blueprint;

use avian2d::prelude::{Collider, LinearVelocity, RigidBody, Sensor};
use bevy::prelude::*;

use entropy::{Entropy, ForkableRng, GlobalEntropy, RngCore};
use track::{Checkpoint, CheckpointTracker, Track, TrackInterior, Wall};

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
            (
                Self::track_last_checkpoint,
                Self::drive_resurfacer,
                Self::resurface_track,
                Self::spawn_resurfacer,
            )
                .chain()
                .in_set(ResurfacerSystems),
        );
        app.register_type::<Resurfacer>()
            .register_type::<TrackResurfacer>();
    }
}

impl ResurfacerPlugin {
    fn track_last_checkpoint(
        tracks: Query<&TrackResurfacer, With<Track>>,
        mut resurfacers: Query<
            (&mut Resurfacer, &CheckpointTracker),
            (Changed<CheckpointTracker>, With<Resurfacer>),
        >,
        checkpoints: Query<&Checkpoint>,
    ) {
        for resurfacer in &tracks {
            let Ok((mut resurfacer, tracker)) = resurfacers.get_mut(**resurfacer) else {
                continue;
            };
            if tracker.len() == 0 {
                continue;
            }
            let Some(last_reached_checkpoint) = tracker.iter().last() else {
                continue;
            };
            let checkpoint = checkpoints
                .get(*last_reached_checkpoint)
                .expect("tracker to track valid Checkpoint entities");
            resurfacer.last_checkpoint_index = checkpoint.index;
        }
    }

    fn drive_resurfacer(
        tracks: Query<(&Track, &TrackResurfacer)>,
        mut resurfacers: Query<(&mut LinearVelocity, &mut Transform, &Resurfacer)>,
    ) {
        for (track, resurfacer) in &tracks {
            let (mut velocity, mut transform, resurfacer) = resurfacers
                .get_mut(**resurfacer)
                .expect("TrackResurfacer to be a valid Resurfacer entity");
            let chunks = track.chunks().collect::<Vec<_>>();
            let index = resurfacer.last_checkpoint_index + 1;
            let next_chunk = chunks
                .get(index)
                .unwrap_or(chunks.get(0).expect("Track to have chunks"));
            let next_checkpoint_position =
                Checkpoint::from_chunk(track, next_chunk.clone(), index).position;
            **velocity = Resurfacer::SPEED
                * (next_checkpoint_position - transform.translation.xy()).normalize();
            let target = Vec3::new(
                next_checkpoint_position.x,
                next_checkpoint_position.y,
                transform.translation.z,
            );
            // TODO: not working?
            transform.look_at(target, Vec3::Z);
        }
    }

    fn resurface_track(
        mut commands: Commands,
        mut resurfacers: Query<
            (&mut CheckpointTracker, &mut Entropy),
            (Changed<CheckpointTracker>, With<Resurfacer>),
        >,
        mut checkpoints: Query<(&Checkpoint, Option<&mut CheckpointObstacles>)>,
        other_colliders: Query<
            &Transform,
            (
                With<Collider>,
                Without<Checkpoint>,
                Without<Track>,
                Without<TrackInterior>,
                Without<Wall>,
            ),
        >,
        obstacles: Query<Entity, With<Obstacle>>,
    ) {
        for (mut tracker, mut entropy) in &mut resurfacers {
            if tracker.len() == 0 {
                continue;
            }
            for checkpoint_entity in tracker.drain() {
                let (checkpoint, checkpoint_obstacles) = checkpoints
                    .get_mut(checkpoint_entity)
                    .expect("tracker to track valid checkpoint entities");
                let mut new_obstacles = vec![];
                // initialize the set of collider positions to the set of other collider positions
                // it will be updated with the positions of spawned stuff positions as we go
                // this will be used to stop us from spawning overlapping entities as well as
                // spawning obstacles on top of drivers
                let mut collider_positions = other_colliders
                    .iter()
                    .map(|transform| transform.translation.xy())
                    .collect::<Vec<_>>();
                while entropy.next_u32() < u32::MAX / 3 {
                    let rand_decimal = entropy.next_u32() as f32 / u32::MAX as f32;
                    let checkpoint_width_position = (rand_decimal - 0.5) * checkpoint.size.x * 0.9;
                    let spawn_position = checkpoint.position
                        + Vec2::from_angle(checkpoint.angle) * checkpoint_width_position;

                    // don't spawn if you collide with something else meaningful
                    if collider_positions
                        .iter()
                        .any(|position| position.distance(spawn_position) < Peg::RADIUS)
                    {
                        continue;
                    }
                    collider_positions.push(spawn_position);
                    let obstacle = commands
                        .spawn((Obstacle::Peg, Peg.bundle(spawn_position)))
                        .id();
                    new_obstacles.push(obstacle);
                }
                if let Some(mut checkpoint_obstacles) = checkpoint_obstacles {
                    for entity in checkpoint_obstacles.drain() {
                        if obstacles.contains(entity) {
                            info!("Despawning obstacle {entity:?}");
                            commands.entity(entity).despawn_recursive();
                        }
                    }
                    checkpoint_obstacles.extend(new_obstacles);
                } else {
                    commands
                        .entity(checkpoint_entity)
                        .insert(CheckpointObstacles::new(new_obstacles));
                }
            }
        }
    }

    fn spawn_resurfacer(
        mut commands: Commands,
        mut entropy: ResMut<GlobalEntropy>,
        tracks: Query<(Entity, &Track), Without<TrackResurfacer>>,
    ) {
        for (track_entity, track) in &tracks {
            let resurfacer = Resurfacer::default();
            let chunk = std::iter::repeat(track)
                .flat_map(|track| {
                    track
                        .chunks()
                        .enumerate()
                        .map(|(index, chunk)| Checkpoint::from_chunk(track, chunk, index))
                })
                .skip(resurfacer.last_checkpoint_index)
                .next()
                .expect("Checkpoints iter to be a ring");
            let resurfacer = commands
                .spawn((
                    resurfacer.bundle(chunk.position, chunk.angle),
                    entropy.fork_rng(),
                ))
                .id();
            commands
                .entity(track_entity)
                .insert(TrackResurfacer(resurfacer));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct ResurfacerSystems;

#[derive(Clone, Debug)]
#[derive(Component, Reflect)]
pub struct Resurfacer {
    last_checkpoint_index: usize,
}

impl Resurfacer {
    const STARTING_CHECKPOINT: usize = 25;
    const WIDTH: f32 = 30.;
    const Z_INDEX: f32 = 15.;
    const SPEED: f32 = 120.;

    fn transform(position: Vec2, angle: f32) -> Transform {
        Transform::from_translation(Vec3::new(position.x, position.y, Self::Z_INDEX))
            .with_rotation(Quat::from_rotation_z(angle))
    }

    pub fn bundle(self, position: Vec2, angle: f32) -> impl Bundle {
        (
            Blueprint::new(self.clone()),
            self,
            Name::new("Resurfacer"),
            CheckpointTracker::default(),
            RigidBody::Kinematic,
            Collider::rectangle(Self::WIDTH, Self::WIDTH),
            Sensor,
            SpatialBundle::from_transform(Self::transform(position, angle)),
        )
    }
}

impl Default for Resurfacer {
    fn default() -> Self {
        Resurfacer {
            last_checkpoint_index: Self::STARTING_CHECKPOINT,
        }
    }
}

#[derive(Debug)]
#[derive(Component, Deref, Reflect)]
pub struct TrackResurfacer(Entity);

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub enum Obstacle {
    #[default]
    Peg,
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Peg;

impl Peg {
    const Z_INDEX: f32 = 12.;
    const RADIUS: f32 = 20.;

    pub fn bundle(self, position: Vec2) -> impl Bundle {
        (
            Blueprint::new(self.clone()),
            self,
            Name::new(format!("Peg Obstacle")),
            RigidBody::Static,
            SpatialBundle::from_transform(Transform::from_xyz(
                position.x,
                position.y,
                Self::Z_INDEX,
            )),
            Collider::circle(Self::RADIUS),
        )
    }
}

#[derive(Debug)]
#[derive(Component, Deref, Reflect)]
pub struct CheckpointObstacles(Vec<Entity>);

impl CheckpointObstacles {
    pub fn new(obstacles: Vec<Entity>) -> Self {
        Self(obstacles)
    }

    pub fn extend(&mut self, obstacles: Vec<Entity>) {
        self.0.extend(obstacles);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Entity> + '_ {
        self.0.drain(..)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use avian2d::{
        prelude::{Physics, PhysicsTime, TimestepMode},
        PhysicsPlugins,
    };
    use bevy::{ecs::system::RunSystemOnce, scene::ScenePlugin};

    use track::{TrackInterior, TrackPlugin};

    use super::*;

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
                Resurfacer::default(),
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
        let (mut _app, _tracker, _track, _) = test_app();

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
