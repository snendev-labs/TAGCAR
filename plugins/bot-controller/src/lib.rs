use avian2d::prelude::{
    LayerMask, LinearVelocity, Rotation, ShapeCaster, ShapeHits, SpatialQueryFilter,
};
use bevy::prelude::*;

use car::{AccelerateAction, CarPhysicsBundle, DrivingSystems, SteerAction};
use entropy::{Entropy, ForkableRng, GlobalEntropy, RngCore};
use laptag::{BombTagIt, CanBeIt, LapTagIt};
use track::{Checkpoint, CheckpointTracker};

pub struct BotControllerPlugin;

impl Plugin for BotControllerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, BotControlSystems.before(DrivingSystems))
            .add_systems(
                Update,
                (Self::compute_goal_positions, Self::decide_bot_controls)
                    .chain()
                    .in_set(BotControlSystems),
            );
    }
}

impl BotControllerPlugin {
    fn compute_goal_positions(
        mut bots: Query<
            (
                Entity,
                &mut BotState,
                &ShapeHits,
                Option<&CheckpointTracker>,
            ),
            With<BotController>,
        >,
        players: Query<
            (
                &Transform,
                &LinearVelocity,
                Option<&LapTagIt>,
                Option<&BombTagIt>,
            ),
            With<CanBeIt>,
        >,
        checkpoints: Query<(Entity, &Checkpoint)>,
    ) {
        for (bot, mut bot_state, shape_hits, tracker) in &mut bots {
            let Ok((bot_transform, bot_velocity, bot_lap_tag, bot_bomb_tag)) = players.get(bot)
            else {
                continue;
            };
            let bot_transform = *bot_transform;
            let bot_velocity = **bot_velocity;
            let has_lap_tag = bot_lap_tag.is_some();
            let has_bomb_tag = bot_bomb_tag.is_some();
            let bot_position = bot_transform.translation.xy();

            const EDGE_NEG: f32 = -std::f32::consts::PI;
            const EDGE_POS: f32 = std::f32::consts::PI;
            const BOTTOM_LEFT_ANGLE: f32 = -5. / 6. * std::f32::consts::PI;
            const BOTTOM_RIGHT_ANGLE: f32 = -std::f32::consts::FRAC_PI_6;
            const TOP_RIGHT_ANGLE: f32 = std::f32::consts::FRAC_PI_6;
            const TOP_LEFT_ANGLE: f32 = 5. / 6. * std::f32::consts::PI;

            let positional_ideal_rotation = match bot_position.to_angle() {
                // bottom straightaway
                BOTTOM_LEFT_ANGLE..=BOTTOM_RIGHT_ANGLE => 0.,
                // top straightaway
                TOP_RIGHT_ANGLE..=TOP_LEFT_ANGLE => std::f32::consts::PI,
                // rounding the bends
                BOTTOM_RIGHT_ANGLE..=TOP_RIGHT_ANGLE
                | EDGE_NEG..=BOTTOM_LEFT_ANGLE
                | TOP_LEFT_ANGLE..=EDGE_POS => {
                    bot_position.to_angle() + std::f32::consts::FRAC_PI_2
                }
                _ => panic!("Vec2::to_angle() returned outside [-pi, +pi]"),
            };

            let facing_direction = if bot_velocity.length() < 0.001 {
                bot_transform.local_x().as_vec3().xy()
            } else {
                bot_velocity.normalize()
            };

            let facing_position_influence =
                bot_transform.translation.xy() + facing_direction * 100.;

            let mut avoid_factor: f32 = 1.;
            let bot_predictive_position = bot_position + bot_velocity.normalize() * 100.;

            // if we have lap tag, recalculate ideal position and rotation and increase avoid factor
            let mut lap_chase_position_influence = None;
            if has_lap_tag && !checkpoints.is_empty() && tracker.is_some() {
                let nearest_checkpoint = checkpoints
                    .iter()
                    .filter(|(checkpoint, _)| !tracker.unwrap().contains(checkpoint))
                    .min_by(|(_, checkpoint1), (_, checkpoint2)| {
                        let distance1 = checkpoint1.position.distance(bot_predictive_position);
                        let distance2 = checkpoint2.position.distance(bot_predictive_position);
                        distance1.total_cmp(&distance2)
                    })
                    .unwrap()
                    .1;
                let origin = nearest_checkpoint.chunk.origin();
                let nearest_point = origin
                    + (bot_position - origin)
                        .project_onto(Vec2::from_angle(nearest_checkpoint.chunk.angle()));
                let target = if nearest_point.distance_squared(nearest_checkpoint.position)
                    > Checkpoint::WIDTH * Checkpoint::WIDTH / 4.
                {
                    nearest_point
                } else {
                    nearest_checkpoint.position
                };

                avoid_factor *= 5.;
                lap_chase_position_influence = Some(target);
            }
            // if we have bomb tag, set avoid factor to negative
            if has_bomb_tag {
                avoid_factor *= -1.;
            }

            // now use raycast to choose how to interpret avoid_factor
            let mut shapecast_position_influence = None;
            let nearest_hit = shape_hits
                .iter()
                // get the first contact, ignoring checkpoints
                .find(|ray_hit| !checkpoints.contains(ray_hit.entity));
            if let Some((transform, _, has_lap, has_bomb)) =
                nearest_hit.and_then(|ray_hit| players.get(ray_hit.entity).ok())
            {
                if has_bomb.is_some() && !has_bomb_tag {
                    // avoid!!!
                    avoid_factor = avoid_factor.abs() * 5.;
                }
                if has_lap.is_some() && !has_lap_tag {
                    // chase!!
                    avoid_factor = -avoid_factor.abs() * 5.;
                }

                shapecast_position_influence = Some(if avoid_factor.is_sign_negative() {
                    transform.translation.xy()
                } else {
                    let target_delta = transform.translation.xy() - bot_position;
                    bot_position + facing_direction * target_delta.length() - target_delta
                });
            } else if nearest_hit.is_some() {
                avoid_factor = avoid_factor.abs();
            } else {
                avoid_factor = 0.;
            }

            // start at 100
            let mut weight = 0.;
            let mut weighted_avg = Vec2::ZERO;

            let mut add_weight = |sample: Vec2, sample_weight: f32| {
                weight += sample_weight;
                weighted_avg = weighted_avg + (sample_weight / weight) * (sample - weighted_avg);
            };
            // influence from what direction you are currently going
            add_weight(facing_position_influence, 20.);
            // influence from what quadrant of the grid you are in
            let track_position_influence =
                bot_position + Vec2::from_angle(positional_ideal_rotation) * 100.;
            add_weight(track_position_influence, 30.);
            if let Some(lap_chase_position_influence) = lap_chase_position_influence {
                add_weight(lap_chase_position_influence, 60.);
            }
            if let Some(shapecast_position_influence) = shapecast_position_influence {
                add_weight(shapecast_position_influence, 40.);
            }
            info!("{bot}: {facing_position_influence} {track_position_influence} {lap_chase_position_influence:?} {shapecast_position_influence:?}");
            let ideal_position = weighted_avg;
            info!("{bot}: {bot_position} -> {ideal_position}");
            let next_state = BotState {
                ideal_position: ideal_position,
                ideal_rotation: (ideal_position - bot_position).to_angle(),
                avoid_factor,
            };
            info!("{bot}: {:?}", *bot_state);
            *bot_state = next_state;
        }
    }

    fn decide_bot_controls(
        mut commands: Commands,
        mut bots: Query<(Entity, &BotState, &Rotation, &mut Entropy), With<BotController>>,
    ) {
        for (car, bot, rotation, mut entropy) in &mut bots {
            let aggression = entropy.next_u32() as f32 / u32::MAX as f32;
            let delta_rotation = (bot.ideal_rotation - rotation.as_radians()).tan().atan();
            info!(
                "{car}: {} <> {} => {delta_rotation} ({})",
                bot.ideal_rotation,
                rotation.as_radians(),
                bot.ideal_rotation - rotation.as_radians()
            );
            let is_marginal_rotation = delta_rotation.abs() < std::f32::consts::PI / 64.;
            let mut steering = if is_marginal_rotation {
                0.
            } else if delta_rotation.is_sign_positive() {
                1.
            } else if delta_rotation.is_sign_negative() {
                -1.
            } else {
                0.
            };
            if delta_rotation.abs() > std::f32::consts::FRAC_PI_2 {
                commands.entity(car).insert(AccelerateAction::Backward);
                steering = -steering;
            } else {
                commands.entity(car).insert(AccelerateAction::Forward);
            }
            commands.entity(car).insert(SteerAction(
                steering * aggression * bot.avoid_factor.abs() / 5.,
            ));
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct BotControlSystems;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
struct BotController;

#[derive(Bundle)]
pub struct BotControllerBundle {
    controller: BotController,
    entropy: Entropy,
    shapecast: ShapeCaster,
    bot: BotState,
}

impl BotControllerBundle {
    pub fn new(entropy: &mut GlobalEntropy) -> Self {
        Self {
            controller: BotController,
            entropy: entropy.fork_rng(),
            shapecast: ShapeCaster::new(CarPhysicsBundle::collider(), Vec2::ZERO, 0., Dir2::X)
                .with_max_time_of_impact(300.)
                .with_ignore_origin_penetration(true)
                .with_query_filter(SpatialQueryFilter::from_mask(
                    LayerMask::ALL & (!Checkpoint::COLLISION_LAYER),
                )),
            bot: BotState::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
struct BotState {
    pub ideal_position: Vec2,
    pub ideal_rotation: f32,
    pub avoid_factor: f32,
}
