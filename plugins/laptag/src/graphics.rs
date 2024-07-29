use std::ops::DerefMut;

use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::prelude::*;
use bevy_asset_loader::prelude::{
    AssetCollection, ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_reactive_blueprints::{Blueprint, BlueprintPlugin, FromBlueprint};

use crate::{BombTagIt, LapTagIt, LapTagSystems};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlueprintPlugin::<Flag, FlagGraphicsBundle>::default())
            .add_plugins(BlueprintPlugin::<Bomb, BombGraphicsBundle>::default())
            .init_state::<LapTagAssetsState>()
            .add_loading_state(
                LoadingState::new(LapTagAssetsState::Loading)
                    .load_collection::<LapTagAssets>()
                    .continue_to_state(LapTagAssetsState::Loaded),
            )
            .add_systems(
                Update,
                (Self::track_score_tags, Self::track_bomb_tags).after(LapTagSystems),
            );
        app.register_type::<Bomb>()
            .register_type::<BombGraphic>()
            .register_type::<Flag>()
            .register_type::<FlagGraphic>();
    }
}

impl GraphicsPlugin {
    fn track_score_tags(
        mut commands: Commands,
        new_bomb_tags: Query<Entity, Added<LapTagIt>>,
        graphics: Query<&FlagGraphic>,
        mut removed_bomb_tags: RemovedComponents<LapTagIt>,
    ) {
        for entity in removed_bomb_tags.read() {
            let Ok(FlagGraphic(graphic)) = graphics.get(entity) else {
                continue;
            };
            commands.entity(*graphic).despawn_recursive();
            commands.entity(entity).remove::<BombGraphic>();
        }
        for entity in &new_bomb_tags {
            let graphic = commands.spawn(Blueprint::new(Flag)).id();
            commands
                .entity(entity)
                .insert(FlagGraphic(graphic))
                .add_child(graphic);
        }
    }

    fn track_bomb_tags(
        mut commands: Commands,
        new_bomb_tags: Query<Entity, Added<BombTagIt>>,
        graphics: Query<&BombGraphic>,
        mut removed_bomb_tags: RemovedComponents<BombTagIt>,
    ) {
        for entity in removed_bomb_tags.read() {
            let Ok(BombGraphic(graphic)) = graphics.get(entity) else {
                continue;
            };
            commands.entity(*graphic).despawn_recursive();
            commands.entity(entity).remove::<BombGraphic>();
        }
        for entity in &new_bomb_tags {
            let graphic = commands.spawn(Blueprint::new(Bomb)).id();
            commands
                .entity(entity)
                .insert(BombGraphic(graphic))
                .add_child(graphic);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[derive(States)]
pub enum LapTagAssetsState {
    #[default]
    Loading,
    Loaded,
}

#[derive(AssetCollection, Resource)]
pub struct LapTagAssets {
    #[asset(path = "textures/flag.png")]
    pub flag: Handle<Image>,
    #[asset(path = "textures/bomb.png")]
    pub bomb: Handle<Image>,
}

#[derive(SystemParam)]
pub struct GraphicsAssetsParams<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    textures: ResMut<'w, LapTagAssets>,
}

#[derive(Clone, Debug, Default)]
#[derive(Reflect)]
struct Flag;

#[derive(Clone, Debug)]
#[derive(Component, Reflect)]
struct FlagGraphic(Entity);

#[derive(Bundle)]
pub struct FlagGraphicsBundle {
    sprite: ColorMesh2dBundle,
}

impl FromBlueprint<Flag> for FlagGraphicsBundle {
    type Params<'w, 's> = GraphicsAssetsParams<'w>;

    fn from_blueprint(_: &Flag, params: &mut StaticSystemParam<Self::Params<'_, '_>>) -> Self {
        let params = params.deref_mut();
        Self {
            sprite: ColorMesh2dBundle {
                material: params.materials.add(ColorMaterial {
                    color: Color::WHITE,
                    texture: Some(params.textures.flag.clone()),
                }),
                mesh: params.meshes.add(Rectangle::new(30., 30.).mesh()).into(),
                transform: Transform::from_translation(Vec3::Z)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, Default)]
#[derive(Reflect)]
struct Bomb;

#[derive(Clone, Debug)]
#[derive(Component, Reflect)]
struct BombGraphic(Entity);

#[derive(Bundle)]
pub struct BombGraphicsBundle {
    sprite: ColorMesh2dBundle,
}

impl FromBlueprint<Bomb> for BombGraphicsBundle {
    type Params<'w, 's> = GraphicsAssetsParams<'w>;

    fn from_blueprint(_: &Bomb, params: &mut StaticSystemParam<Self::Params<'_, '_>>) -> Self {
        let params = params.deref_mut();
        Self {
            sprite: ColorMesh2dBundle {
                material: params.materials.add(ColorMaterial {
                    color: Color::WHITE,
                    texture: Some(params.textures.bomb.clone()),
                }),
                mesh: params.meshes.add(Rectangle::new(30., 30.).mesh()).into(),
                transform: Transform::from_translation(Vec3::Z)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
                ..Default::default()
            },
        }
    }
}
