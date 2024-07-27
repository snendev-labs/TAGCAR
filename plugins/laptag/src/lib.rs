use std::marker::PhantomData;

use avian2d::prelude::CollisionStarted;
use bevy::prelude::*;
use bevy::{app::PluginGroupBuilder, ecs::system::EntityCommand, reflect::GetTypeRegistration};

use track::{CheckpointTracker, LapComplete};

#[cfg(feature = "graphics")]
mod particles;
#[cfg(feature = "graphics")]
pub use particles::*;

pub trait TagIt {
    fn finish_lap() -> impl EntityCommand;

    #[cfg(feature = "graphics")]
    type Effect: Component;

    #[cfg(feature = "graphics")]
    fn spawn_effects(position: Vec2) -> impl bevy::ecs::world::Command {
        move |world: &mut World| {
            // let mut effects =
            //     world.query_filtered::<(&mut EffectSpawner, &mut Transform), With<Self::Effect>>();
            // let (mut effect, mut transform) = effects.single_mut(world);
            // transform.translation.x = position.x;
            // transform.translation.y = position.y;
            // effect.reset();
        }
    }
}

pub struct LapTagPlugins;

impl PluginGroup for LapTagPlugins {
    fn build(self) -> PluginGroupBuilder {
        let builder = PluginGroupBuilder::start::<Self>();
        #[cfg(feature = "graphics")]
        let builder = builder.add(ParticlesPlugin);
        builder
            .add(LapTagPlugin::<ScoreTagIt>::default())
            .add(LapTagPlugin::<BombTagIt>::default())
    }
}

#[derive(Default)]
pub struct LapTagPlugin<Tag> {
    marker: PhantomData<Tag>,
}

impl<Tag> Plugin for LapTagPlugin<Tag>
where
    Tag: Component + Default + GetTypeRegistration + TagIt + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::transfer_tag,
                Self::reset_lap_on_tag,
                Self::complete_laps,
            )
                .chain()
                .in_set(LapTagSystems),
        );
        app.register_type::<Score>()
            .register_type::<Tag>()
            .register_type::<CanBeIt>();
    }
}

impl<Tag> LapTagPlugin<Tag>
where
    Tag: TagIt + Component,
{
    fn transfer_tag(
        mut commands: Commands,
        mut collisions: EventReader<CollisionStarted>,
        tag_its: Query<Entity, With<Tag>>,
        can_be_its: Query<Entity, (With<CanBeIt>, Without<Tag>)>,
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
        }
    }

    fn reset_lap_on_tag(
        mut lap_trackers: Query<&mut CheckpointTracker>,
        new_tag_its: Query<Entity, Added<Tag>>,
        mut removed_tag_its: RemovedComponents<Tag>,
    ) {
        for entity in new_tag_its.iter().chain(removed_tag_its.read()) {
            let Ok(mut tracker) = lap_trackers.get_mut(entity) else {
                continue;
            };
            tracker.clear();
        }
    }

    fn complete_laps(
        mut commands: Commands,
        mut completed_laps: EventReader<LapComplete>,
        racers: Query<&Transform, (With<Tag>, With<CheckpointTracker>)>,
    ) {
        for lap in completed_laps.read() {
            if let Ok(transform) = racers.get(lap.racer) {
                #[cfg(feature = "graphics")]
                commands.add(Tag::spawn_effects(transform.translation.xy()));
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

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct ScoreTagIt;

impl TagIt for ScoreTagIt {
    #[cfg(feature = "graphics")]
    type Effect = ConfettiParticles;

    fn finish_lap() -> impl EntityCommand {
        |entity: Entity, world: &mut World| {
            let mut score = world
                .get_mut::<Score>(entity)
                .expect("ScoreTagIt command should only be fired for valid `Score`ing entities");
            **score += 1;
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct BombTagIt;

impl TagIt for BombTagIt {
    #[cfg(feature = "graphics")]
    type Effect = ExplosionParticles;

    fn finish_lap() -> impl EntityCommand {
        |entity: Entity, world: &mut World| {
            world.entity_mut(entity).despawn_recursive();
        }
    }
}
