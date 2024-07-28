use std::{marker::PhantomData, time::Duration};

use avian2d::prelude::{CollisionStarted, Physics};
use bevy::{
    app::PluginGroupBuilder, ecs::system::EntityCommand, prelude::*, reflect::GetTypeRegistration,
    utils::EntityHashSet,
};

use track::{CheckpointTracker, LapComplete};

pub trait TagIt {
    fn finish_lap() -> impl EntityCommand;
}

pub struct LapTagPlugins;

impl PluginGroup for LapTagPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(LapsPlugin)
            .add(TagPlugin::<LapTagIt>::default())
            .add(TagPlugin::<BombTagIt>::default())
    }
}

pub struct LapsPlugin;

impl Plugin for LapsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::tick_immunity, Self::handle_tags)
                .chain()
                .in_set(LapTagSystems),
        );
        app.register_type::<Score>()
            .register_type::<TagImmunity>()
            .register_type::<CanBeIt>();
    }
}

impl LapsPlugin {
    fn handle_tags(
        mut commands: Commands,
        mut lap_trackers: Query<&mut CheckpointTracker>,
        new_tag_its: Query<Entity, Or<(Added<LapTagIt>, Added<BombTagIt>)>>,
        mut removed_lap_tag_its: RemovedComponents<LapTagIt>,
        mut removed_bomb_tag_its: RemovedComponents<BombTagIt>,
    ) {
        // start by assuming we will remove all entities that lost tags
        let mut entities_to_remove = removed_lap_tag_its
            .read()
            .chain(removed_bomb_tag_its.read())
            .collect::<EntityHashSet<_>>();

        for entity in &new_tag_its {
            // if an entity was going to lose a tag, don't do that anymore
            // it will be handled here instead
            entities_to_remove.remove(&entity);
            // also attach some immunity
            commands.entity(entity).insert(TagImmunity::default());
            // now clear or insert a new tracker
            if let Ok(mut tracker) = lap_trackers.get_mut(entity) {
                tracker.clear();
            } else {
                commands.entity(entity).insert(CheckpointTracker::default());
            }
        }

        // now cleanup the remaining entities to remove
        for entity in entities_to_remove {
            if lap_trackers.contains(entity) {
                commands.entity(entity).remove::<CheckpointTracker>();
            }
        }
    }

    fn tick_immunity(
        mut commands: Commands,
        mut timers: Query<(Entity, &mut TagImmunity)>,
        time: Res<Time<Physics>>,
    ) {
        for (entity, mut timer) in &mut timers {
            if timer.tick(time.delta()) {
                commands.entity(entity).remove::<TagImmunity>();
            }
        }
    }
}

#[derive(Default)]
pub struct TagPlugin<Tag> {
    marker: PhantomData<Tag>,
}

impl<Tag> Plugin for TagPlugin<Tag>
where
    Tag: Component + Default + GetTypeRegistration + TagIt + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<TagEvent>().add_systems(
            Update,
            (Self::transfer_tag, Self::complete_laps)
                .chain()
                .in_set(LapTagSystems),
        );
        app.register_type::<Tag>();
    }
}

impl<Tag> TagPlugin<Tag>
where
    Tag: TagIt + Component,
{
    fn transfer_tag(
        mut commands: Commands,
        mut collisions: EventReader<CollisionStarted>,
        tag_its: Query<Entity, (With<Tag>, Without<TagImmunity>)>,
        can_be_its: Query<Entity, (With<CanBeIt>, Without<Tag>)>,
        mut tags: EventWriter<TagEvent>,
    ) where
        Tag: Default,
    {
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
            commands.entity(it_entity).remove::<Tag>();
            commands.entity(tagged_entity).insert(Tag::default());
            tags.send(TagEvent {
                prev_it: it_entity,
                next_it: tagged_entity,
            });
        }
    }

    fn complete_laps(
        mut commands: Commands,
        mut completed_laps: EventReader<LapComplete>,
        racers: Query<Entity, (With<Tag>, With<CheckpointTracker>)>,
    ) {
        for lap in completed_laps.read() {
            if racers.contains(lap.racer) {
                commands.entity(lap.racer).add(Tag::finish_lap());
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct LapTagSystems;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Score(u32);

impl Score {
    pub fn new(num: u32) -> Self {
        Score(num)
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct CanBeIt;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Deref, Reflect)]
pub struct TagImmunity(Duration);

impl Default for TagImmunity {
    fn default() -> Self {
        Self(Duration::from_secs(2))
    }
}

impl TagImmunity {
    fn tick(&mut self, delta: Duration) -> bool {
        self.0 = self.0.saturating_sub(delta);
        self.0.is_zero()
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct LapTagIt;

impl TagIt for LapTagIt {
    fn finish_lap() -> impl EntityCommand {
        |entity: Entity, world: &mut World| {
            let mut score = world
                .get_mut::<Score>(entity)
                .expect("LapTagIt command should only be fired for valid `Score`ing entities");
            **score += 1;
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct BombTagIt;

impl TagIt for BombTagIt {
    fn finish_lap() -> impl EntityCommand {
        |entity: Entity, world: &mut World| {
            world.entity_mut(entity).despawn_recursive();
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Event, Reflect)]
pub struct TagEvent {
    pub prev_it: Entity,
    pub next_it: Entity,
}
