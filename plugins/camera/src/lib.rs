use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_dolly::prelude::{Dolly, Position, Rig, Smooth};

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::spawn_camera.in_set(GameCameraSystems::Spawn))
            .add_systems(
                Update,
                (Self::camera_tracking, Dolly::<GameCamera>::update_2d_active)
                    .chain()
                    .in_set(GameCameraSystems::Track),
            );
        app.register_type::<GameCamera>();
    }
}

impl GameCameraPlugin {
    fn spawn_camera(mut commands: Commands) {
        commands.spawn((
            GameCamera,
            Rig::builder()
                .with(Position::default())
                .with(Smooth::new_position(0.8))
                .build(),
            Name::new("Game Camera"),
            Camera2dBundle::default(),
        ));
    }

    fn camera_tracking(
        tracker: Query<(&CameraTracker, &Transform, Option<&LinearVelocity>)>,
        mut rigs: Query<&mut Rig>,
    ) {
        let Ok((CameraTracker { bounds }, transform, velocity)) = tracker.get_single() else {
            return;
        };
        let velocity = velocity.map(|velocity| **velocity).unwrap_or(Vec2::ZERO);
        let mut rig = rigs.single_mut();
        let camera_driver = rig.driver_mut::<Position>();

        camera_driver.position = Vec3::new(
            (transform.translation.x + velocity.x)
                .max(bounds.min.x)
                .min(bounds.max.x),
            (transform.translation.y + velocity.y)
                .max(bounds.min.y)
                .min(bounds.max.y),
            0.,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum GameCameraSystems {
    Spawn,
    Track,
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct GameCamera;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct CameraTracker {
    bounds: Rect,
}

impl CameraTracker {
    pub fn rect(min: Vec2, max: Vec2) -> Self {
        Self {
            bounds: Rect::from_corners(min, max),
        }
    }
}
