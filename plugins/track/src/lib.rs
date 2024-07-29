use std::f32::consts::{FRAC_PI_2, PI};

use avian2d::prelude::{Collider, CollisionLayers, CollisionStarted, LayerMask, RigidBody, Sensor};
use bevy::color::palettes;
use bevy::prelude::*;
use bevy::utils::EntityHashSet;
use bevy_reactive_blueprints::Blueprint;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use graphics::*;

pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "graphics")]
        app.add_plugins(GraphicsPlugin);
        app.add_event::<LapComplete>();
        app.add_systems(
            Update,
            (Self::spawn_checkpoints, Self::track_checkpoints)
                .chain()
                .in_set(TrackSystems),
        );
        app.register_type::<Track>()
            .register_type::<TrackInterior>()
            .register_type::<Checkpoint>()
            .register_type::<Checkpoints>()
            .register_type::<CheckpointTracker>()
            .register_type::<LapComplete>();
    }
}

impl TrackPlugin {
    fn spawn_checkpoints(
        mut commands: Commands,
        tracks: Query<(Entity, &Track), Without<Checkpoints>>,
    ) {
        for (entity, track) in &tracks {
            let mut checkpoints = vec![];
            let mut walls = vec![];
            let mut chunks = track.chunks().collect::<Vec<_>>();
            for (index, chunk) in chunks.iter().enumerate() {
                let checkpoint = commands
                    .spawn(Checkpoint::from_chunk(track, chunk.clone(), index).bundle())
                    .id();
                checkpoints.push(checkpoint);
            }
            // complete the ring by adding the first element to the end
            chunks.push(chunks.get(0).unwrap().clone());
            // iterate through all edges in the ring
            for chunk_pair in chunks.windows(2) {
                let chunk1 = &chunk_pair[0];
                let chunk2 = &chunk_pair[1];
                let wall = commands
                    .spawn(Wall::between_chunks(track, chunk1.clone(), chunk2.clone()).bundle())
                    .id();
                walls.push(wall);
            }

            commands.entity(entity).insert(Checkpoints(checkpoints));
            commands.entity(entity).insert(Walls(walls));
        }
    }

    fn track_checkpoints(
        mut collisions: EventReader<CollisionStarted>,
        mut completed_laps: EventWriter<LapComplete>,
        mut trackers: Query<&mut CheckpointTracker>,
        checkpoints: Query<Entity, With<Checkpoint>>,
    ) {
        let num_checkpoints = checkpoints.iter().count();
        for CollisionStarted(entity1, entity2) in collisions.read() {
            let tracker_entity = if trackers.contains(*entity1) {
                *entity1
            } else if trackers.contains(*entity2) {
                *entity2
            } else {
                continue;
            };
            let checkpoint_entity = if checkpoints.contains(*entity1) {
                *entity1
            } else if checkpoints.contains(*entity2) {
                *entity2
            } else {
                continue;
            };
            let mut collision = trackers.get_mut(tracker_entity).unwrap();
            let checkpoint = checkpoints.get(checkpoint_entity).unwrap();
            if let Some(lap_complete) =
                collision.reach_checkpoint(tracker_entity, checkpoint, num_checkpoints)
            {
                completed_laps.send(lap_complete);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct TrackSystems;

#[derive(Clone, Debug)]
#[derive(Component, Reflect)]
pub struct Track {
    half_length: f32,
    radius: f32,
    thickness: f32,
    subdivisions_per_chunk: usize,
}

impl Default for Track {
    fn default() -> Self {
        Self::new(1600., 800., 600., 12)
    }
}

impl Track {
    pub const ASPHALT: Color = Color::Srgba(palettes::css::DIM_GRAY);
    pub const GRASS: Color = Color::Srgba(palettes::css::FOREST_GREEN);

    pub fn new(half_length: f32, radius: f32, thickness: f32, subdivisions: usize) -> Self {
        Track {
            half_length,
            radius,
            thickness,
            subdivisions_per_chunk: subdivisions,
        }
    }

    pub fn bundle(self) -> impl Bundle {
        (
            Blueprint::new(self.clone()),
            SpatialBundle::from_transform(Transform::from_rotation(Quat::from_rotation_z(
                FRAC_PI_2,
            ))),
            Name::new(format!("Track")),
            self,
        )
    }

    pub fn half_length(&self) -> f32 {
        self.half_length
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn thickness(&self) -> f32 {
        self.thickness
    }

    pub fn interior_radius(&self) -> f32 {
        self.radius - self.thickness
    }

    pub fn chunks(&self) -> impl Iterator<Item = TrackChunk> + '_ {
        // iterate through the "sides" of the track and flat_map to a list of subdivisions
        let x = self.half_length - self.radius;
        let separation = x * 2. / self.subdivisions_per_chunk as f32;
        let top_chunk_range = (0..=self.subdivisions_per_chunk).map(move |index| {
            TrackChunk::new(
                Vec2::new(x - index as f32 * separation, 0.),
                std::f32::consts::FRAC_PI_2,
            )
        });

        let bottom_chunk_range = (0..=self.subdivisions_per_chunk).rev().map(move |index| {
            TrackChunk::new(
                Vec2::new(x - index as f32 * separation, 0.),
                -std::f32::consts::FRAC_PI_2,
            )
        });

        // draw N subdivisions on a half circle
        // x = r * cos(theta), y = r * sin(theta)
        // on left side, theta in range (pi/2, 3pi/2)
        // I don't know why this center works
        let left_chunk_range = (1..self.subdivisions_per_chunk).map(move |index| {
            let theta = FRAC_PI_2 + PI * index as f32 / self.subdivisions_per_chunk as f32;
            TrackChunk::new(Vec2::new(-x, 0.), theta)
        });
        // on right side, theta in range (-pi/2, pi/2)
        let right_chunk_range = (1..self.subdivisions_per_chunk).map(move |index| {
            let theta = -FRAC_PI_2 + PI * index as f32 / self.subdivisions_per_chunk as f32;
            TrackChunk::new(Vec2::new(x, 0.), theta)
        });
        top_chunk_range
            .chain(left_chunk_range)
            .chain(bottom_chunk_range)
            .chain(right_chunk_range)
    }
}

#[derive(Clone, Debug, Default)]
#[derive(Reflect)]
pub struct TrackChunk {
    chunk_origin: Vec2,
    chunk_border_angle: f32,
}

impl TrackChunk {
    pub fn new(chunk_origin: Vec2, chunk_border_angle: f32) -> Self {
        Self {
            chunk_origin,
            chunk_border_angle,
        }
    }

    pub fn origin(&self) -> Vec2 {
        self.chunk_origin
    }

    pub fn angle(&self) -> f32 {
        self.chunk_border_angle
    }
}

#[derive(Clone, Debug, Default)]
#[derive(Component, Reflect)]
pub struct TrackInterior {
    radius: f32,
    half_length: f32,
}

impl TrackInterior {
    const Z_INDEX: f32 = 5.;

    pub fn from_track(track: &Track) -> Self {
        let radius = track.interior_radius();
        let half_length = track.half_length;
        Self {
            radius,
            half_length,
        }
    }

    pub fn bundle(self) -> impl Bundle {
        (
            Blueprint::new(self.clone()),
            Name::new(format!("Track interior")),
            RigidBody::Static,
            Collider::capsule(self.radius, self.half_length),
            Sensor,
            SpatialBundle::from_transform(
                Transform::from_xyz(0., 0., Self::Z_INDEX)
                    .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
            ),
            self,
        )
    }
}

#[derive(Clone, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Checkpoint {
    pub index: usize,
    pub size: Vec2,
    pub position: Vec2,
    pub chunk: TrackChunk,
}

impl Checkpoint {
    pub const WIDTH: f32 = 4.;
    const Z_INDEX: f32 = 10.;
    pub const COLLISION_LAYER: LayerMask = LayerMask(1 << 5);

    pub fn from_chunk(track: &Track, chunk: TrackChunk, index: usize) -> Self {
        let size = Vec2::new(track.thickness, Self::WIDTH);
        let track_center_offset = track.radius() / 2. + track.interior_radius() / 2.;

        Checkpoint {
            index,
            size,
            position: chunk.chunk_origin
                + Vec2::from_angle(chunk.chunk_border_angle) * track_center_offset,
            chunk,
        }
    }

    pub fn transform(&self) -> Transform {
        Transform::from_translation(Vec3::new(self.position.x, self.position.y, Self::Z_INDEX))
            .with_rotation(Quat::from_rotation_z(self.chunk.angle()))
    }

    pub fn bundle(self) -> impl Bundle {
        (
            Blueprint::new(self.clone()),
            Name::new(format!("Checkpoint {}", self.index)),
            RigidBody::Static,
            Collider::rectangle(self.size.x, self.size.y),
            Sensor,
            SpatialBundle::from_transform(self.transform()),
            CollisionLayers::new(Self::COLLISION_LAYER, LayerMask::ALL),
            self,
        )
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct Checkpoints(Vec<Entity>);

#[derive(Debug, Default)]
#[derive(Component, Deref, Reflect)]
pub struct CheckpointTracker {
    checkpoints: EntityHashSet<Entity>,
}

impl CheckpointTracker {
    pub fn reach_checkpoint(
        &mut self,
        self_entity: Entity,
        checkpoint: Entity,
        expected_total: usize,
    ) -> Option<LapComplete> {
        self.checkpoints.insert(checkpoint);
        if self.checkpoints.len() >= expected_total {
            self.clear();
            Some(LapComplete { racer: self_entity })
        } else {
            None
        }
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Entity> + '_ {
        self.checkpoints.drain()
    }

    pub fn clear(&mut self) {
        self.checkpoints.clear();
    }
}

#[derive(Debug)]
#[derive(Deref, Event, Reflect)]
pub struct LapComplete {
    pub racer: Entity,
}

#[derive(Clone, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Wall {
    pub size: Vec2,
    pub position: Vec2,
    pub angle: f32,
}

impl Wall {
    const Z_INDEX: f32 = 15.;
    const THICKNESS: f32 = 10.;

    pub fn between_chunks(track: &Track, chunk1: TrackChunk, chunk2: TrackChunk) -> Self {
        let vertex1 =
            chunk1.chunk_origin + Vec2::from_angle(chunk1.chunk_border_angle) * track.radius();
        let vertex2 =
            chunk2.chunk_origin + Vec2::from_angle(chunk2.chunk_border_angle) * track.radius();
        let size = Vec2::new(vertex1.distance(vertex2) + 0.2, Self::THICKNESS);
        Wall {
            size,
            position: (vertex1 + vertex2) / 2.,
            angle: (chunk1.chunk_border_angle + chunk2.chunk_border_angle + std::f32::consts::PI)
                / 2.,
        }
    }

    pub fn transform(&self) -> Transform {
        Transform::from_translation(Vec3::new(self.position.x, self.position.y, Self::Z_INDEX))
            .with_rotation(Quat::from_rotation_z(self.angle))
    }

    pub fn bundle(self) -> impl Bundle {
        (
            Blueprint::new(self.clone()),
            Name::new("Wall"),
            RigidBody::Static,
            Collider::rectangle(self.size.x, self.size.y),
            SpatialBundle::from_transform(self.transform()),
            self,
        )
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct Walls(Vec<Entity>);

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use avian2d::{
        prelude::{Physics, PhysicsTime, TimestepMode},
        PhysicsPlugins,
    };
    use bevy::{ecs::system::RunSystemOnce, scene::ScenePlugin};

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
        let (e1, e2, e3) = app.world_mut().run_system_once(spawn_track_and_tracker);
        (app, e1, e2, e3)
    }

    fn spawn_track_and_tracker(mut commands: Commands) -> (Entity, Entity, Entity) {
        let track = Track::default();
        let interior = TrackInterior::from_track(&track);
        let tracker = commands
            .spawn((
                CheckpointTracker::default(),
                RigidBody::Kinematic,
                Collider::rectangle(10., 10.),
                SpatialBundle::from_transform(Transform::from_xyz(track.half_length, 0., 0.)),
            ))
            .id();
        let interior = commands.spawn(interior.bundle()).id();
        let track = commands.spawn(track.bundle()).id();
        (tracker, track, interior)
    }

    #[test]
    fn test_lap_completion() {
        let (mut app, tracker, track, _) = test_app();

        let track = app.world_mut().get::<Track>(track).unwrap().clone();
        for (index, chunk) in track.clone().chunks().enumerate() {
            let events = app.world_mut().resource::<Events<LapComplete>>();
            let mut reader = events.get_reader();
            assert!(reader.read(events).find(|lap| ***lap == tracker).is_none());
            let reached_checkpoints = app.world_mut().get::<CheckpointTracker>(tracker).unwrap();
            assert_eq!(reached_checkpoints.len(), index);
            let mut transform = app.world_mut().get_mut::<Transform>(tracker).unwrap();
            *transform = Checkpoint::from_chunk(&track, chunk, index).transform();
            app.update();
            app.update();
        }

        let events = app.world_mut().resource::<Events<LapComplete>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).find(|lap| ***lap == tracker).is_some());
        let reached_checkpoints = app.world_mut().get::<CheckpointTracker>(tracker).unwrap();
        assert_eq!(reached_checkpoints.len(), 0);
    }
}
