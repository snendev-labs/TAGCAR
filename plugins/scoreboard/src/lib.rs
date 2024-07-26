use bevy::prelude::*;
use sickle_ui::prelude::*;

pub struct ScoreBoardPlugin;

impl Plugin for ScoreBoardPlugin {
    fn build(&self, app: &mut App) {}
}

fn spawn_scoreboard(mut commands: Commands) {
    commands
        .ui_builder(UiRoot)
        .column(|column| column.style().width(Val::Percent(15.)))
        .style()
        .width(Val::Percent(100.));
}
