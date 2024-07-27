use bevy::prelude::*;
use laptag::Score;
use sickle_ui::{prelude::*, ui_commands::SetTextExt};

pub struct ScoreboardPlugin;

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (Self::attach_scoreboard, Self::update_scoreboard));
    }
}

impl ScoreboardPlugin {
    fn attach_scoreboard(
        mut commands: Commands,
        scoreboard_query: Query<Entity, (With<Scoreboard>, Without<ScoreboardUI>)>,
    ) {
        for scoreboard_entity in &scoreboard_query {
            commands.entity(scoreboard_entity).insert(ScoreboardUI);
        }
    }

    fn update_scoreboard(
        mut commands: Commands,
        scores_query: Query<(&CarName, Ref<Score>)>,
        scoreboard_query: Query<(Entity, &ScoreboardUI)>,
    ) {
        if scores_query.iter().any(|(_, score)| score.is_changed()) {
            let mut scores: Vec<(String, u32)> = scores_query
                .iter()
                .map(|(car_name, score)| ((*car_name).0.clone(), (*score).get()))
                .collect::<Vec<(String, u32)>>();

            // b.cmp(a) in order to get reverse sorting with largest scores first
            scores.sort_by(|a, b| b.1.cmp(&a.1));

            if let Ok((entity, _scoreboard)) = scoreboard_query.get_single() {
                commands.entity(entity).despawn_descendants();
                commands.ui_builder(entity).generate_scoreboard_ui(scores);
            }
        }
    }
}

pub trait UiScoreboardExt {
    fn generate_scoreboard_ui(&mut self, sorted_scores: Vec<(String, u32)>) -> UiBuilder<Entity>;
}

impl UiScoreboardExt for UiBuilder<'_, Entity> {
    fn generate_scoreboard_ui(&mut self, sorted_scores: Vec<(String, u32)>) -> UiBuilder<Entity> {
        self.column(|column| {
            for score in sorted_scores.into_iter() {
                column
                    .row(|row| {
                        row.label(LabelConfig::default())
                            .entity_commands()
                            .set_text(score.0, None);
                        row.label(LabelConfig::default())
                            .entity_commands()
                            .set_text(score.1.to_string(), None);
                    })
                    .style()
                    .justify_content(JustifyContent::SpaceBetween)
                    .width(Val::Percent(100.));
            }
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component)]
pub struct Scoreboard;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component)]
pub struct ScoreboardUI;

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct CarName(pub String);
