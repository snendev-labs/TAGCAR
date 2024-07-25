use std::ops::DerefMut;

use bevy::color::palettes;
use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::prelude::*;

use bevy_reactive_blueprints::{AsChild, BlueprintPlugin, FromBlueprint};

use crate::{Checkpoint, Track, TrackInterior};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlueprintPlugin::<Track, TrackGraphicsBundle, AsChild>::default())
            .add_plugins(BlueprintPlugin::<
                TrackInterior,
                TrackInteriorGraphicsBundle,
                AsChild,
            >::default())
            .add_plugins(BlueprintPlugin::<
                Checkpoint,
                CheckpointGraphicsBundle,
                AsChild,
            >::default());
    }
}
#[derive(SystemParam)]
pub struct GraphicsAssetsParams<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
}

#[derive(Bundle)]
pub struct TrackGraphicsBundle {
    sprite: ColorMesh2dBundle,
}

impl TrackGraphicsBundle {
    pub fn from_track(
        track: &Track,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        Self {
            sprite: ColorMesh2dBundle {
                material: materials.add(Track::ASPHALT),
                mesh: meshes
                    .add(Capsule2d::new(track.radius, track.half_length).mesh())
                    .into(),
                ..Default::default()
            },
        }
    }
}

impl FromBlueprint<Track> for TrackGraphicsBundle {
    type Params<'w, 's> = GraphicsAssetsParams<'w>;

    fn from_blueprint(track: &Track, params: &mut StaticSystemParam<Self::Params<'_, '_>>) -> Self {
        let params = params.deref_mut();
        Self::from_track(track, params.meshes.as_mut(), params.materials.as_mut())
    }
}

#[derive(Bundle)]
pub struct TrackInteriorGraphicsBundle {
    sprite: ColorMesh2dBundle,
}

impl TrackInteriorGraphicsBundle {
    pub fn new(
        interior: &TrackInterior,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        Self {
            sprite: ColorMesh2dBundle {
                material: materials.add(Track::GRASS),
                mesh: meshes
                    .add(Capsule2d::new(interior.radius, interior.half_length).mesh())
                    .into(),
                ..Default::default()
            },
        }
    }
}

impl FromBlueprint<TrackInterior> for TrackInteriorGraphicsBundle {
    type Params<'w, 's> = GraphicsAssetsParams<'w>;

    fn from_blueprint(
        interior: &TrackInterior,
        params: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        let params = params.deref_mut();
        Self::new(interior, params.meshes.as_mut(), params.materials.as_mut())
    }
}

#[derive(Bundle)]
pub struct CheckpointGraphicsBundle {
    sprite: ColorMesh2dBundle,
}

impl CheckpointGraphicsBundle {
    pub fn new(
        checkpoint: &Checkpoint,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        Self {
            sprite: ColorMesh2dBundle {
                material: materials.add(Color::Srgba(palettes::css::WHITE_SMOKE.with_alpha(0.1))),
                mesh: meshes
                    .add(Rectangle::new(checkpoint.size.x, checkpoint.size.y).mesh())
                    .into(),
                ..Default::default()
            },
        }
    }
}

impl FromBlueprint<Checkpoint> for CheckpointGraphicsBundle {
    type Params<'w, 's> = GraphicsAssetsParams<'w>;

    fn from_blueprint(
        checkpoint: &Checkpoint,
        params: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        let params = params.deref_mut();
        Self::new(
            checkpoint,
            params.meshes.as_mut(),
            params.materials.as_mut(),
        )
    }
}
