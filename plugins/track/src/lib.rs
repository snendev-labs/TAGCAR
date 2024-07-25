// TODO: Track plugin using mariokart-style checkpoints and a means
// of determining lap completion with on-the-fly lap markers

use std::f32::consts::{FRAC_PI_2, PI};

use avian2d::prelude::{Collider, CollisionStarted, RigidBody, Sensor};
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
            for (position, angle) in track.checkpoints() {
                let entity = commands
                    .spawn(Checkpoint::bundle(position, angle, track.thickness))
                    .id();
                checkpoints.push(entity);
            }

            commands.entity(entity).insert(Checkpoints(checkpoints));
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
        Self::new(600., 300., 220., 10)
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

    pub fn checkpoints(&self) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        // we want to go from
        //    ---------
        //   /
        //  /
        //  \
        //   \
        //    ---------
        // so we iterate through the "sides" of the track
        // and flat_map each chunk's subdivisions
        // etc

        // TrackChunk::Top
        let y = self.radius;
        let x = self.half_length - self.radius;
        let separation = x * 2. / self.subdivisions_per_chunk as f32;
        let top_chunk_range = (0..=self.subdivisions_per_chunk).map(move |index| {
            (
                Vec2::new(x - index as f32 * separation, y - self.thickness / 2.),
                0.,
            )
        });

        let bottom_chunk_range = (0..=self.subdivisions_per_chunk).map(move |index| {
            (
                Vec2::new(x - index as f32 * separation, -y + self.thickness / 2.),
                0.,
            )
        });

        // draw N subdivisions on a half circle
        // x = r * cos(theta), y = r * sin(theta)
        // on left side, theta in range (pi/2, 3pi/2)
        let track_center = (self.radius + self.thickness / 4.) / 2.;
        let left_chunk_range = (1..self.subdivisions_per_chunk).map(move |index| {
            let theta = FRAC_PI_2 + PI * index as f32 / self.subdivisions_per_chunk as f32;
            (
                Vec2::new(-x + track_center * theta.cos(), track_center * theta.sin()),
                theta - FRAC_PI_2,
            )
        });
        // on right side, theta in range (-pi/2, pi/2)
        let right_chunk_range = (1..self.subdivisions_per_chunk).map(move |index| {
            let theta = -FRAC_PI_2 + PI * index as f32 / self.subdivisions_per_chunk as f32;
            (
                Vec2::new(x + track_center * theta.cos(), track_center * theta.sin()),
                theta - FRAC_PI_2,
            )
        });
        top_chunk_range
            .chain(left_chunk_range)
            .chain(bottom_chunk_range)
            .chain(right_chunk_range)
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
        let radius = track.radius - track.thickness;
        let half_length = track.half_length;
        Self {
            radius,
            half_length,
        }
    }

    pub fn bundle(self) -> impl Bundle {
        (
            Blueprint::new(self.clone()),
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

#[derive(Debug, Default)]
#[derive(Component, Reflect)]
pub struct Checkpoint {
    pub size: Vec2,
}

impl Checkpoint {
    const WIDTH: f32 = 4.;
    const Z_INDEX: f32 = 10.;

    pub fn bundle(position: Vec2, angle: f32, thickness: f32) -> CheckpointBundle {
        let x = Self::WIDTH;
        let y = thickness * 1.1;
        CheckpointBundle {
            checkpoint: Checkpoint {
                size: Vec2::new(x, y),
            },
            rigid_body: RigidBody::Static,
            collider: Collider::rectangle(x, y),
            sensor: Sensor,
            spatial: SpatialBundle::from_transform(
                Transform::from_translation(Vec3::new(position.x, position.y, Self::Z_INDEX))
                    .with_rotation(Quat::from_rotation_z(angle)),
            ),
        }
    }
}

#[derive(Bundle)]
pub struct CheckpointBundle {
    checkpoint: Checkpoint,
    rigid_body: RigidBody,
    collider: Collider,
    sensor: Sensor,
    spatial: SpatialBundle,
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct Checkpoints(Vec<Entity>);

#[derive(Debug, Default)]
#[derive(Component, Reflect)]
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
            Some(LapComplete(self_entity))
        } else {
            None
        }
    }

    pub fn contains(&self, checkpoint: Entity) -> bool {
        self.checkpoints.contains(&checkpoint)
    }

    pub fn clear(&mut self) {
        self.checkpoints.clear();
    }
}

#[derive(Debug)]
#[derive(Deref, Event, Reflect)]
pub struct LapComplete(Entity);

#[cfg(test)]
mod tests {
    use super::*;
    use avian2d::PhysicsPlugins;
    use bevy::{ecs::system::RunSystemOnce, scene::ScenePlugin};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ScenePlugin,
            PhysicsPlugins::default(),
        ));
        app.add_plugins(TrackPlugin);
        app.world_mut().run_system_once(spawn_track_and_tracker);
        app
    }

    fn spawn_track_and_tracker(mut commands: Commands) {
        let track = Track::default();
        let interior = TrackInterior::from_track(&track);
        commands.spawn((
            CheckpointTracker::default(),
            Collider::rectangle(10., 10.),
            SpatialBundle::from_transform(Transform::from_xyz(track.half_length, 0., 0.)),
        ));
        commands.spawn(interior.bundle());
        commands.spawn(track.bundle());
    }

    #[test]
    fn test_lap_completion() {
        let mut app = test_app();

        app.update();

        unimplemented!()
    }
}
