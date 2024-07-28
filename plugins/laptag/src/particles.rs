use bevy::prelude::*;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_particle_sprites.in_set(ParticleSystems));
    }
}

impl ParticlesPlugin {}

fn animate_particle_sprites(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlas,
    )>,
) {
    for (entity, indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                commands.entity(entity).despawn(); // Change to remove sprite components, not despawn entity
                return;
            } else {
                atlas.index + 1
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct ParticleSystems;

pub trait ParticleSprite {
    fn generate_sprite_bundle(
        asset_server: Res<AssetServer>,
        texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ) -> (SpriteBundle, TextureAtlas, AnimationIndices);
}

#[derive(Component, Reflect)]
pub struct ConfettiSprite;

impl ParticleSprite for ConfettiSprite {
    fn generate_sprite_bundle(
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ) -> (SpriteBundle, TextureAtlas, AnimationIndices) {
        let confetti_texture = asset_server.load("sprites/confetti.png");
        let confetti_layout = TextureAtlasLayout::from_grid(UVec2::splat(512), 8, 8, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(confetti_layout);

        let animation_indices = AnimationIndices { first: 1, last: 8 };
        (
            SpriteBundle {
                transform: Transform::default(),
                texture: confetti_texture,
                ..default()
            },
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
            animation_indices,
        )
    }
}

#[derive(Copy, Clone, Debug)]
#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Clone, Debug)]
#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

// #[derive(Clone, Copy, Debug)]
// #[derive(Component)]
// pub enum AnimationRepetition {
//     Finite(usize),
//     Infinite,
// }
