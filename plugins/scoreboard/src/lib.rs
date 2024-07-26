use bevy::prelude::*;
use laptag::Score;
use sickle_ui::{prelude::*, ui_commands::SetTextExt};

pub struct ScoreBoardPlugin;

impl Plugin for ScoreBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scoreboard)
            .add_systems(Update, update_scoreboard);
    }
}

fn spawn_scoreboard(mut commands: Commands) {
    commands.ui_builder(UiRoot).column(|column| {
        column
            .style()
            .position_type(PositionType::Absolute)
            .right(Val::Px(20.))
            .top(Val::Px(20.))
            .height(Val::Auto)
            .width(Val::Px(200.))
            .padding(UiRect::all(Val::Px(5.)))
            .background_color(Color::srgba(0.6, 0.6, 0.6, 0.2));
        column
            .label(LabelConfig::default())
            .entity_commands()
            .set_text("Scoreboard", None);
        column.insert(Scoreboard::default());
    });
}

fn update_scoreboard(
    mut commands: Commands,
    scores_query: Query<(Entity, &CarName, Ref<Score>)>,
    scoreboard_query: Query<(Entity, &Scoreboard)>,
) {
    if scores_query.iter().any(|tuple| tuple.2.is_changed()) {
        let mut scores: Vec<(String, u32)> = scores_query
            .iter()
            .map(|(entity, car_name, score)| ((*car_name).0.clone(), (*score).get()))
            .collect::<Vec<(String, u32)>>();

        // b.cmp(a) in order to get reverse sorting with largest scores first
        scores.sort_by(|a, b| b.1.cmp(&a.1));

        if let Ok((entity, _scoreboard)) = scoreboard_query.get_single() {
            commands.entity(entity).despawn_descendants();
            commands.ui_builder(entity).column(|column| {
                for score in scores.into_iter() {
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
            });
        }
    }
}

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct CarName(pub String);

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component)]
pub struct Scoreboard;

#[cfg(test)]
mod test {}
