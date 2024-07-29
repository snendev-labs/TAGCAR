use bevy::color::palettes;
use bevy::ecs::system::StaticSystemParam;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use bevy_reactive_blueprints::{AsChild, BlueprintPlugin, FromBlueprint};

use crate::{Car, CarBlueprint, Wheel};

pub struct CarGraphicsPlugin;

impl Plugin for CarGraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BlueprintPlugin::<CarBlueprint, CarGraphicsBundle, AsChild>::default())
            .add_plugins(BlueprintPlugin::<Wheel, WheelGraphicsBundle, AsChild>::default());
    }
}

#[derive(Bundle)]
pub struct CarGraphicsBundle {
    pub shape: MaterialMesh2dBundle<ColorMaterial>,
}

impl CarGraphicsBundle {
    pub fn new(shape: MaterialMesh2dBundle<ColorMaterial>) -> Self {
        CarGraphicsBundle { shape }
    }
}

impl FromBlueprint<CarBlueprint> for CarGraphicsBundle {
    type Params<'w, 's> = (ResMut<'w, Assets<Mesh>>, ResMut<'w, Assets<ColorMaterial>>);

    fn from_blueprint(
        _blueprint: &CarBlueprint,
        params: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        CarGraphicsBundle {
            shape: MaterialMesh2dBundle {
                mesh: params.0.add(Rectangle::new(Car::LENGTH, Car::WIDTH)).into(),
                material: params.1.add(Color::from(palettes::css::AQUA)),
                ..Default::default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct WheelGraphicsBundle {
    pub shape: MaterialMesh2dBundle<ColorMaterial>,
}

impl WheelGraphicsBundle {
    pub fn new(shape: MaterialMesh2dBundle<ColorMaterial>) -> Self {
        WheelGraphicsBundle { shape }
    }
}

impl FromBlueprint<Wheel> for WheelGraphicsBundle {
    type Params<'w, 's> = (ResMut<'w, Assets<Mesh>>, ResMut<'w, Assets<ColorMaterial>>);

    fn from_blueprint(
        _blueprint: &Wheel,
        params: &mut StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Self {
        WheelGraphicsBundle {
            shape: MaterialMesh2dBundle {
                mesh: params
                    .0
                    .add(Rectangle::new(Wheel::LENGTH, Wheel::WIDTH))
                    .into(),
                material: params.1.add(Color::from(palettes::css::BLUE)),
                ..Default::default()
            },
        }
    }
}
