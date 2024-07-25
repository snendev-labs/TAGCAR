use avian2d::prelude::CollisionStarted;
use bevy::prelude::*;
use track::{CheckpointTracker, LapComplete};

pub struct LapTagPlugin;

impl Plugin for LapTagPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::transfer_tag,
                Self::reset_lap_on_tag,
                Self::score_completed_laps,
            )
                .chain()
                .in_set(LapTagSystems),
        );
        app.register_type::<Score>()
            .register_type::<TagIt>()
            .register_type::<CanBeIt>();
    }
}

impl LapTagPlugin {
    fn transfer_tag(
        mut commands: Commands,
        mut collisions: EventReader<CollisionStarted>,
        tag_its: Query<Entity, With<TagIt>>,
        can_be_its: Query<Entity, (With<CanBeIt>, Without<TagIt>)>,
    ) {
        for CollisionStarted(entity1, entity2) in collisions.read() {
            let entity1_is_it = tag_its.contains(*entity1);
            let entity2_is_it = tag_its.contains(*entity2);
            let entity1_can_be_it = can_be_its.contains(*entity1);
            let entity2_can_be_it = can_be_its.contains(*entity2);
            let (it_entity, tagged_entity) = if entity1_is_it && !entity2_is_it && entity2_can_be_it
            {
                (*entity1, *entity2)
            } else if entity2_is_it && !entity1_is_it && entity1_can_be_it {
                (*entity2, *entity1)
            } else {
                continue;
            };
            commands.entity(it_entity).remove::<TagIt>();
            commands.entity(tagged_entity).insert(TagIt);
        }
    }

    fn reset_lap_on_tag(
        mut lap_trackers: Query<&mut CheckpointTracker>,
        new_tag_its: Query<Entity, Added<TagIt>>,
        mut removed_tag_its: RemovedComponents<TagIt>,
    ) {
        for entity in new_tag_its.iter().chain(removed_tag_its.read()) {
            let Ok(mut tracker) = lap_trackers.get_mut(entity) else {
                continue;
            };
            tracker.clear();
        }
    }

    fn score_completed_laps(
        mut completed_laps: EventReader<LapComplete>,
        mut scores: Query<&mut Score, (With<TagIt>, With<CheckpointTracker>)>,
    ) {
        for lap in completed_laps.read() {
            let Ok(mut score) = scores.get_mut(lap.racer) else {
                continue;
            };
            **score += 1;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct LapTagSystems;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Score(u32);

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct TagIt;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct CanBeIt;
