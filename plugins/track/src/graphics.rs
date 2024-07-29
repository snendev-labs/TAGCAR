use std::ops::DerefMut;

use bevy::color::palettes;
use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::prelude::*;
use bevy_asset_loader::prelude::{
    AssetCollection, ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_reactive_blueprints::{AsChild, Blueprint, BlueprintPlugin, FromBlueprint};

use crate::{Checkpoint, CheckpointTracker, Track, TrackInterior, Wall};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlueprintPlugin::<Track, TrackGraphicsBundle, AsChild>::default())
            .add_plugins(BlueprintPlugin::<
                TrackInterior,
                TrackInteriorGraphicsBundle,
                AsChild,
            >::default())
            .add_plugins(BlueprintPlugin::<TrackLogo, TrackLogoGraphicsBundle, AsChild>::default())
            .add_plugins(BlueprintPlugin::<
                Checkpoint,
                CheckpointGraphicsBundle,
                AsChild,
            >::default())
            .add_plugins(BlueprintPlugin::<Wall, WallGraphicsBundle, AsChild>::default())
            .init_state::<TrackAssetsState>()
            .add_loading_state(
                LoadingState::new(TrackAssetsState::Loading)
                    .load_collection::<TrackAssets>()
                    .continue_to_state(TrackAssetsState::Loaded),
            )
            .add_systems(Startup, Self::initialize_color_materials)
            .add_systems(Update, Self::track_checkpoint_colors);
    }
}

impl GraphicsPlugin {
    fn initialize_color_materials(
        mut commands: Commands,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        let normal = materials.add(CheckpointColors::NORMAL_COLOR);
        let highlighted = materials.add(CheckpointColors::HIGHLIGHTED_COLOR);
        commands.insert_resource(CheckpointColors {
            normal,
            highlighted,
        });
    }

    fn track_checkpoint_colors(
        mut checkpoints: Query<(&Parent, &mut Handle<ColorMaterial>), With<CheckpointGraphics>>,
        colors: Res<CheckpointColors>,
        updated_trackers: Query<Entity, Changed<CheckpointTracker>>,
        trackers_to_highlight: Query<Option<&CheckpointTracker>, With<CheckpointHighlightTracker>>,
        mut removed_trackers: RemovedComponents<CheckpointTracker>,
    ) {
        let Some(tracker) = updated_trackers
            .iter()
            .chain(removed_trackers.read())
            .find_map(|entity| trackers_to_highlight.get(entity).ok())
        else {
            return;
        };
        for (parent, mut handle) in &mut checkpoints {
            if tracker.is_some_and(|tracker| tracker.contains(&**parent)) {
                *handle = colors.highlighted.clone();
            } else {
                *handle = colors.normal.clone();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[derive(States)]
pub enum TrackAssetsState {
    #[default]
    Loading,
    Loaded,
}

#[derive(AssetCollection, Resource)]
pub struct TrackAssets {
    #[cfg_attr(not(target_arch = "wasm32"), asset(path = "textures/logo.png"))]
    pub logo: Handle<Image>,
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct CheckpointHighlightTracker;

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
    logo: Blueprint<TrackLogo>,
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
            logo: Blueprint::new(TrackLogo(interior.clone())),
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

#[derive(Clone, Debug, Default)]
#[derive(Reflect)]
pub struct TrackLogo(TrackInterior);

#[derive(Bundle)]
pub struct TrackLogoGraphicsBundle {
    sprite: ColorMesh2dBundle,
}

impl TrackLogoGraphicsBundle {
    pub fn new(
        logo: &TrackLogo,
        texture: Handle<Image>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        Self {
            sprite: ColorMesh2dBundle {
                material: materials.add(ColorMaterial {
                    color: Color::WHITE,
                    texture: Some(texture),
                }),
                mesh: meshes
                    .add(Rectangle::new(logo.0.half_length * 0.8, logo.0.radius * 1.6).mesh())
                    .into(),
                transform: Transform::from_translation(Vec3::Z)
                    .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
                ..Default::default()
            },
        }
    }
}

impl FromBlueprint<TrackLogo> for TrackLogoGraphicsBundle {
    type Params<'w, 's> = (GraphicsAssetsParams<'w>, Res<'w, TrackAssets>);

    fn from_blueprint(
        logo: &TrackLogo,
        params: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        let params = params.deref_mut();
        Self::new(
            logo,
            params.1.logo.clone(),
            params.0.meshes.as_mut(),
            params.0.materials.as_mut(),
        )
    }
}

#[derive(Clone, Copy)]
#[derive(Component, Reflect)]
pub struct CheckpointGraphics;

#[derive(Debug)]
#[derive(Resource, Reflect)]
pub struct CheckpointColors {
    normal: Handle<ColorMaterial>,
    highlighted: Handle<ColorMaterial>,
}

impl CheckpointColors {
    const NORMAL_COLOR: Color = Color::Srgba(Srgba {
        alpha: 0.1,
        ..palettes::css::WHITE_SMOKE
    });
    const HIGHLIGHTED_COLOR: Color = Color::Srgba(Srgba {
        alpha: 1.,
        ..palettes::css::SKY_BLUE
    });
}

#[derive(Bundle)]
pub struct CheckpointGraphicsBundle {
    sprite: ColorMesh2dBundle,
    marker: CheckpointGraphics,
}

impl CheckpointGraphicsBundle {
    pub fn new(
        checkpoint: &Checkpoint,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        Self {
            sprite: ColorMesh2dBundle {
                material: materials.add(CheckpointColors::NORMAL_COLOR),
                mesh: meshes
                    .add(Rectangle::new(checkpoint.size.x, checkpoint.size.y).mesh())
                    .into(),
                ..Default::default()
            },
            marker: CheckpointGraphics,
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

#[derive(Bundle)]
pub struct WallGraphicsBundle {
    sprite: ColorMesh2dBundle,
}

impl WallGraphicsBundle {
    pub fn new(
        wall: &Wall,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        Self {
            sprite: ColorMesh2dBundle {
                material: materials.add(Color::Srgba(palettes::css::ROSY_BROWN)),
                mesh: meshes
                    .add(Rectangle::new(wall.size.x, wall.size.y).mesh())
                    .into(),
                ..Default::default()
            },
        }
    }
}

impl FromBlueprint<Wall> for WallGraphicsBundle {
    type Params<'w, 's> = GraphicsAssetsParams<'w>;

    fn from_blueprint(wall: &Wall, params: &mut StaticSystemParam<Self::Params<'_, '_>>) -> Self {
        let params = params.deref_mut();
        Self::new(wall, params.meshes.as_mut(), params.materials.as_mut())
    }
}
