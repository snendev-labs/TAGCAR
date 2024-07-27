use bevy::prelude::*;
use seldom_pixel::prelude::*;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(HanabiPlugin);
        // app.add_systems(Startup, Self::initialize_effects);
        // app.register_type::<ConfettiParticles>()
        //     .register_type::<ExplosionParticles>();
    }
}

impl ParticlesPlugin {
    // fn initialize_effects(mut commands: Commands, mut effect_assets: ResMut<Assets<EffectAsset>>) {
    //     let effect = effect_assets.add(ConfettiParticles::effect_asset());
    //     commands.spawn((
    //         ConfettiParticles,
    //         ParticleEffectBundle {
    //             effect: ParticleEffect::new(effect).with_z_layer_2d(Some(2.)),
    //             ..Default::default()
    //         },
    //         Name::new("Confetti Particle Effect"),
    //     ));
    //     let effect = effect_assets.add(ExplosionParticles::effect_asset());
    //     commands.spawn((
    //         ExplosionParticles,
    //         ParticleEffectBundle {
    //             effect: ParticleEffect::new(effect).with_z_layer_2d(Some(2.)),
    //             ..Default::default()
    //         },
    //         Name::new("Explosion Particle Effect"),
    //     ));
    // }
}

#[derive(Component, Reflect)]
pub struct ConfettiParticles;

// impl ConfettiParticles {
//     fn effect_asset() -> EffectAsset {
//         let mut gradient = Gradient::new();
//         gradient.add_key(0.0, Vec4::new(0.5, 0.5, 1.0, 1.0));
//         gradient.add_key(1.0, Vec4::new(0.5, 0.5, 1.0, 0.2));

//         let writer = ExprWriter::new();

//         let age = writer.lit(0.).expr();
//         let init_age = SetAttributeModifier::new(Attribute::AGE, age);

//         let lifetime = writer.lit(5.).expr();
//         let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

//         let init_pos = SetPositionCircleModifier {
//             center: writer.lit(Vec3::ZERO).expr(),
//             axis: writer.lit(Vec3::Z).expr(),
//             radius: writer.lit(0.05).expr(),
//             dimension: ShapeDimension::Surface,
//         };

//         let init_vel = SetVelocityCircleModifier {
//             center: writer.lit(Vec3::ZERO).expr(),
//             axis: writer.lit(Vec3::Z).expr(),
//             speed: writer.lit(0.1).expr(),
//         };

//         let mut module = writer.finish();

//         let round = RoundModifier::constant(&mut module, 2.0 / 3.0);

//         let spawner = Spawner::once(30.0.into(), false);
//         EffectAsset::new(vec![4096], spawner, module)
//             .with_name("Confetti Particles")
//             .init(init_pos)
//             .init(init_vel)
//             .init(init_age)
//             .init(init_lifetime)
//             .render(SizeOverLifetimeModifier {
//                 gradient: Gradient::constant(Vec2::splat(0.02)),
//                 screen_space_size: false,
//             })
//             .render(ColorOverLifetimeModifier { gradient })
//             .render(round)
//     }
// }

#[derive(Component, Reflect)]
pub struct ExplosionParticles;

// impl ExplosionParticles {
//     fn effect_asset() -> EffectAsset {
//         let mut gradient = Gradient::new();
//         gradient.add_key(0.0, Vec4::new(1.0, 0.1, 0.1, 1.0));
//         gradient.add_key(1.0, Vec4::new(0.5, 0.1, 0.1, 0.5));

//         let writer = ExprWriter::new();

//         let age = writer.lit(0.).expr();
//         let init_age = SetAttributeModifier::new(Attribute::AGE, age);

//         let lifetime = writer.lit(5.).expr();
//         let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

//         let init_pos = SetPositionCircleModifier {
//             center: writer.lit(Vec3::ZERO).expr(),
//             axis: writer.lit(Vec3::Z).expr(),
//             radius: writer.lit(0.05).expr(),
//             dimension: ShapeDimension::Surface,
//         };

//         let init_vel = SetVelocityCircleModifier {
//             center: writer.lit(Vec3::ZERO).expr(),
//             axis: writer.lit(Vec3::Z).expr(),
//             speed: writer.lit(0.1).expr(),
//         };

//         let mut module = writer.finish();

//         let round = RoundModifier::constant(&mut module, 2.0 / 3.0);

//         let spawner = Spawner::once(30.0.into(), false);
//         EffectAsset::new(vec![4096], spawner, module)
//             .with_name("Explosion Particles")
//             .init(init_pos)
//             .init(init_vel)
//             .init(init_age)
//             .init(init_lifetime)
//             .render(SizeOverLifetimeModifier {
//                 gradient: Gradient::constant(Vec2::splat(0.02)),
//                 screen_space_size: false,
//             })
//             .render(ColorOverLifetimeModifier { gradient })
//             .render(round)
//     }
// }
