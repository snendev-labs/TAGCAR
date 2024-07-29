use bevy::{color::palettes, prelude::*};
use bot_controller::BotControllerSystems;
use controller::CarControlSystems;
use sickle_ui::prelude::*;

use car::Car;
use entropy::GlobalEntropy;
use laptag::{LapTagSystems, Score};
use track::Track;

use crate::spawn_cars;

pub struct GameLoopPlugin;

impl Plugin for GameLoopPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            LapTagSystems.run_if(not(resource_exists::<GameOver>)),
        );
        app.add_systems(
            Update,
            (Self::restart_game, Self::handle_gameover)
                .chain()
                .before(CarControlSystems)
                .before(BotControllerSystems),
        );
    }
}

impl GameLoopPlugin {
    fn restart_game(
        mut commands: Commands,
        restart_button: Query<&Interaction, With<RestartButton>>,
        cars: Query<Entity, With<Car>>,
        track: Query<&Track>,
        gameover_ui: Query<Entity, With<GameoverUI>>,
        mut entropy: ResMut<GlobalEntropy>,
    ) {
        let Ok(interaction) = restart_button.get_single() else {
            return;
        };
        if !matches!(interaction, Interaction::Pressed) {
            return;
        }
        for car in &cars {
            commands.entity(car).despawn_recursive();
        }
        for entity in &gameover_ui {
            commands.entity(entity).despawn_recursive();
        }
        spawn_cars(&mut commands, track.single(), entropy.as_mut());
    }

    fn handle_gameover(
        mut commands: Commands,
        mut destroyed_players: RemovedComponents<Player>,
        scores: Query<(&Score, Option<&Player>)>,
    ) {
        // whether the player won or not
        let game_result = if destroyed_players.read().count() > 0 {
            Some(false)
        } else {
            scores
                .iter()
                .map(|(score, player)| (**score, player.is_some()))
                .max_by(|(score1, _), (score2, _)| score1.cmp(score2))
                .filter(|(score, _)| *score >= 5)
                .map(|(_, is_player)| is_player)
        };
        if let Some(is_game_won) = game_result {
            commands
                .ui_builder(UiRoot)
                .column(|column| {
                    column
                        .column(|column| {
                            column
                                .label(LabelConfig {
                                    label: "Game Over!".to_string(),
                                    ..Default::default()
                                })
                                .style()
                                .font_size(48.);
                            column
                                .label(LabelConfig {
                                    label: format!(
                                        "YOU {}",
                                        if is_game_won { "WIN" } else { "LOSE" }
                                    ),
                                    ..Default::default()
                                })
                                .style()
                                .font_size(96.);
                            column
                                .container((RestartButton, ButtonBundle::default()), |builder| {
                                    builder
                                        .label(LabelConfig {
                                            label: "Restart".to_string(),
                                            ..Default::default()
                                        })
                                        .style()
                                        .font_size(64.);
                                })
                                .style()
                                .width(Val::Px(300.))
                                .justify_content(JustifyContent::Center)
                                .align_items(AlignItems::Center)
                                .border(UiRect::all(Val::Px(4.)))
                                .border_color(Color::BLACK)
                                .background_color(Color::Srgba(palettes::css::BLUE_VIOLET));
                        })
                        .style()
                        .height(Val::Auto)
                        .padding(UiRect::all(Val::Px(20.)))
                        .row_gap(Val::Px(10.))
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center)
                        .border(UiRect::all(Val::Px(8.)))
                        .border_color(Color::BLACK)
                        .background_color(Color::srgb(0.2, 0.2, 0.2));
                })
                .insert((GameoverUI, Name::new("Gameover UI")))
                .style()
                .height(Val::Percent(100.))
                .width(Val::Percent(100.))
                .background_color(Color::srgba(0.3, 0.3, 0.3, 0.1))
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Player;

#[derive(Clone, Copy, Debug)]
#[derive(Resource, Reflect)]
pub struct GameOver;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component)]
pub struct GameoverUI;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component)]
pub struct RestartButton;
