use bevy::{
    color::palettes::css::{BLUE, RED},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use car::{CarBlueprint, CarBundle, CarGraphicsBundle, CarPhysicsBundle};

fn main() {
    let mut app = App::new();

    let default_plugins = DefaultPlugins.build().set(WindowPlugin {
        primary_window: Some(Window {
            resizable: true,
            resolution: bevy::window::WindowResolution::new(600.0, 900.0),
            ..Default::default()
        }),
        ..Default::default()
    });

    app.add_plugins(default_plugins)
        .add_systems(Startup, (setup_camera, setup_surface, spawn_car));
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_surface(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::default()).into(),
        transform: Transform::default().with_scale(Vec3::splat(10000.)),
        material: materials.add(Color::from(BLUE)),
        ..default()
    });
}

fn spawn_car(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        CarBundle::default(),
        CarPhysicsBundle::default(),
        CarGraphicsBundle {
            shape: MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(1.0, 1.0))),
                material: materials.add(Color::from(RED)),
                transform: Transform::from_xyz(0., 0., 0.1),
                ..default()
            },
        },
    ));
}
