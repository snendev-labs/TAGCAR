use bevy::prelude::*;
use bevy_dolly::prelude::{Dolly, Position, Rig, Smooth};

use entropy::*;

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::spawn_camera.in_set(GameCameraSystems::Spawn))
            .configure_sets(
                Update,
                GameCameraSystems::Shake.after(GameCameraSystems::Track),
            )
            .add_systems(
                Update,
                (Self::camera_tracking, Dolly::<GameCamera>::update_2d_active)
                    .chain()
                    .in_set(GameCameraSystems::Track),
            )
            .add_systems(
                Update,
                (Self::shake_camera).in_set(GameCameraSystems::Shake),
            );
        app.register_type::<GameCamera>();
    }
}

impl GameCameraPlugin {
    fn spawn_camera(mut commands: Commands, mut entropy: ResMut<GlobalEntropy<WyRand>>) {
        commands.spawn((
            GameCamera,
            Rig::builder()
                .with(Position::default())
                .with(Smooth::new_position(0.8))
                .build(),
            Name::new("Game Camera"),
            entropy.fork_rng(),
            Camera2dBundle::default(),
        ));
    }

    fn camera_tracking(tracker: Query<(&CameraTracker, &Transform)>, mut rigs: Query<&mut Rig>) {
        let Ok((CameraTracker { bounds }, transform)) = tracker.get_single() else {
            return;
        };
        let mut rig = rigs.single_mut();
        let camera_driver = rig.driver_mut::<Position>();

        camera_driver.position = Vec3::new(
            transform.translation.x.max(bounds.min.x).min(bounds.max.x),
            transform.translation.y.max(bounds.min.y).min(bounds.max.y),
            0.,
        );
    }

    fn shake_camera(mut rigs: Query<(&mut Rig, &mut Entropy)>) {
        let (mut rig, mut entropy) = rigs.single_mut();
        let camera_driver = rig.driver_mut::<Position>();
        let shake =
            Vec3::new(entropy.next_u32() as f32, entropy.next_u32() as f32, 0.).normalize_or_zero();
        info!("{shake}");
        camera_driver.translate(shake * 100.);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum GameCameraSystems {
    Spawn,
    Track,
    Shake,
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
