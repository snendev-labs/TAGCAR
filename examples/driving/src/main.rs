use bevy::prelude::*;

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
        .add_systems(Startup, setup_camera);

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_surface(mut commands: Commands) {
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::default()).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: materials.add(Color::from(PURPLE)),
        ..default()
    });
}
