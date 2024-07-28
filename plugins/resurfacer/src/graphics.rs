use std::ops::DerefMut;

use bevy::color::palettes;
use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::prelude::*;

use bevy_reactive_blueprints::{AsChild, BlueprintPlugin, FromBlueprint};

use crate::{Peg, Resurfacer};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlueprintPlugin::<
            Resurfacer,
            ResurfacerGraphicsBundle,
            AsChild,
        >::default())
            .add_plugins(BlueprintPlugin::<Peg, PegGraphicsBundle, AsChild>::default());
    }
}
#[derive(SystemParam)]
pub struct GraphicsAssetsParams<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
}

#[derive(Bundle)]
pub struct ResurfacerGraphicsBundle {
    sprite: ColorMesh2dBundle,
}

impl ResurfacerGraphicsBundle {
    pub fn from_resurfacer(
        _resurfacer: &Resurfacer,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        Self {
            sprite: ColorMesh2dBundle {
                material: materials.add(Color::Srgba(palettes::css::BLUE)),
                mesh: meshes
                    .add(Cuboid::from_length(Resurfacer::WIDTH).mesh())
                    .into(),
                ..Default::default()
            },
        }
    }
}

impl FromBlueprint<Resurfacer> for ResurfacerGraphicsBundle {
    type Params<'w, 's> = GraphicsAssetsParams<'w>;

    fn from_blueprint(
        resurfacer: &Resurfacer,
        params: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        let params = params.deref_mut();
        Self::from_resurfacer(
            resurfacer,
            params.meshes.as_mut(),
            params.materials.as_mut(),
        )
    }
}

#[derive(Bundle)]
pub struct PegGraphicsBundle {
    sprite: ColorMesh2dBundle,
}

impl FromBlueprint<Peg> for PegGraphicsBundle {
    type Params<'w, 's> = GraphicsAssetsParams<'w>;

    fn from_blueprint(_: &Peg, params: &mut StaticSystemParam<Self::Params<'_, '_>>) -> Self {
        let params = params.deref_mut();
        Self {
            sprite: ColorMesh2dBundle {
                material: params
                    .materials
                    .add(Color::Srgba(palettes::css::SADDLE_BROWN)),
                mesh: params.meshes.add(Circle::new(Peg::RADIUS).mesh()).into(),
                ..Default::default()
            },
        }
    }
}
