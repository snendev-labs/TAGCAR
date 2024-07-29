use bevy::{color::palettes, prelude::*};

use entropy::GlobalEntropy;
use laptag::LapTagAssets;
use scoreboard::Scoreboard;
use track::{Track, TrackAssets, TrackInterior};

use tagcar::{spawn_cars, Player, TagcarPlugins};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TagcarPlugins);

    app.add_systems(Startup, spawn_loading_ui);

    let run_condition = resource_exists::<TrackAssets>
        .and_then(resource_exists::<LapTagAssets>)
        .and_then(run_once());
    #[cfg(feature = "audio")]
    let run_condition = resource_exists::<bg_music::BgMusicAssets>
        .and_then(resource_exists::<audio_fx::AudioFxAssets>)
        .and_then(run_condition);
    app.add_systems(Update, (spawn_game, despawn_ui).run_if(run_condition));
    app.add_systems(Update, die);
    app.run();
}

#[derive(Component)]
struct LoadingUI;

fn spawn_loading_ui(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Loading UI"),
            LoadingUI,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|builder| {
            builder.spawn(TextBundle::from_section(
                "Loading...",
                TextStyle {
                    font_size: 128.0,
                    color: Color::srgb(0.02, 0.02, 0.1),
                    ..Default::default()
                },
            ));
        });
}

fn spawn_game(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut entropy: ResMut<GlobalEntropy>,
) {
    let track = Track::default();
    spawn_cars(&mut commands, &track, entropy.as_mut());
    commands.spawn((
        Name::new("Background"),
        ColorMesh2dBundle {
            mesh: meshes
                .add(Rectangle::new(track.radius(), track.half_length()))
                .into(),
            material: materials.add(Color::Srgba(palettes::css::DARK_SLATE_GREY)),
            transform: Transform::from_translation(Vec3::NEG_Z).with_scale(Vec3::splat(5.)),
            ..Default::default()
        },
    ));
    let interior = TrackInterior::from_track(&track);
    commands.spawn(interior.bundle());
    commands.spawn(track.bundle());
    commands.spawn(Scoreboard);
}

fn despawn_ui(mut commands: Commands, ui_roots: Query<Entity, With<LoadingUI>>) {
    for entity in &ui_roots {
        commands.entity(entity).despawn_recursive();
    }
}

fn die(
    mut commands: Commands,
    inputs: Res<ButtonInput<KeyCode>>,
    players: Query<Entity, With<Player>>,
) {
    if inputs.just_pressed(KeyCode::KeyQ) {
        for entity in &players {
            commands.entity(entity).despawn_recursive();
        }
    }
}
