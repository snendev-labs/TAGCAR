use avian2d::prelude::{
    LayerMask, LinearVelocity, Rotation, ShapeCaster, ShapeHits, SpatialQueryFilter,
};
use bevy::prelude::*;

use car::{AccelerateAction, CarPhysicsBundle, DrivingSystems, SteerAction};
use entropy::{Entropy, ForkableRng, GlobalEntropy, RngCore};
use laptag::{BombTagIt, CanBeIt, LapTagIt};
use resurfacer::Peg;
use track::{Checkpoint, CheckpointTracker, Wall};

pub struct BotControllerPlugin;

impl Plugin for BotControllerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, BotControllerSystems.before(DrivingSystems))
            .add_systems(
                Update,
                (
                    Self::compute_goals,
                    Self::decide_bot_controls,
                    #[cfg(feature = "gizmos")]
                    Self::render_bot_gizmos,
                )
                    .chain()
                    .in_set(BotControllerSystems),
            );
        app.register_type::<BotController>()
            .register_type::<Goal>()
            .register_type::<BotGoals>();
    }
}

impl BotControllerPlugin {
    fn compute_goals(
        mut bots: Query<
            (
                Entity,
                &LinearVelocity,
                &mut BotGoals,
                &ShapeHits,
                Option<&CheckpointTracker>,
            ),
            With<BotController>,
        >,
        players: Query<(Entity, &Transform, Option<&LapTagIt>, Option<&BombTagIt>), With<CanBeIt>>,
        obstacles: Query<&Transform, Or<(With<Wall>, With<Peg>)>>,
        checkpoints: Query<(Entity, &Checkpoint)>,
    ) {
        for (bot, bot_velocity, mut goals, shape_hits, tracker) in &mut bots {
            let Ok((_, bot_transform, bot_lap_tag, bot_bomb_tag)) = players.get(bot) else {
                continue;
            };

            let bot_transform = *bot_transform;
            let bot_position = bot_transform.translation.xy();
            let bot_has_lap_tag = bot_lap_tag.is_some();
            let bot_has_bomb_tag = bot_bomb_tag.is_some();

            // establish some starting priorities for the bot
            let mut new_goals = BotGoals::default();
            new_goals
                .0
                .push(Goal::max_speed(bot_transform, **bot_velocity));
            new_goals.0.push(Goal::follow_track(bot_position));
            if bot_has_lap_tag && !bot_has_bomb_tag {
                if let Some(tracker) = tracker {
                    new_goals.0.push(Goal::reach_checkpoints(
                        bot_position,
                        **bot_velocity,
                        checkpoints.iter(),
                        tracker,
                    ));
                }
            }

            // figure out who to chase
            for (entity, transform, has_lap_tag, _) in players.iter() {
                if bot == entity {
                    continue;
                }
                if has_lap_tag.is_some() && !bot_has_lap_tag {
                    new_goals
                        .0
                        .push(Goal::chase(transform.translation.xy(), 12.));
                }
            }

            // now shapecast and figure out what to avoid
            if let Some(hit) = shape_hits.iter().next() {
                if let Some((_, transform, _, has_bomb_tag)) = players.get(hit.entity).ok() {
                    if has_bomb_tag.is_some() && !bot_has_bomb_tag {
                        new_goals
                            .0
                            .push(Goal::avoid(transform.translation.xy(), hit.time_of_impact));
                    }
                } else if let Ok(transform) = obstacles.get(hit.entity) {
                    new_goals
                        .0
                        .push(Goal::avoid(transform.translation.xy(), hit.time_of_impact));
                }
            }

            *goals = new_goals;
        }
    }

    fn decide_bot_controls(
        mut commands: Commands,
        mut bots: Query<
            (
                Entity,
                &BotGoals,
                &Transform,
                &Rotation,
                &LinearVelocity,
                &mut Entropy,
            ),
            With<BotController>,
        >,
    ) {
        use std::f32::consts::{FRAC_2_PI, FRAC_PI_2, PI};
        for (car, goals, transform, rotation, velocity, mut entropy) in &mut bots {
            let bot_position = transform.translation.xy();
            let ideal_position = weighted_avg(
                goals
                    .0
                    .iter()
                    .map(|goal| goal.to_influence(bot_position, **velocity)),
            );
            let ideal_rotation = (ideal_position - bot_position).to_angle();
            let aggression = entropy.next_u32() as f32 / u32::MAX as f32;
            let delta_rotation = (ideal_rotation - rotation.as_radians()).tan().atan();

            let is_marginal_rotation = delta_rotation.abs() < PI / 64.;
            let mut steer_signum = if is_marginal_rotation {
                0.
            } else if delta_rotation.is_sign_positive() {
                1.
            } else if delta_rotation.is_sign_negative() {
                -1.
            } else {
                0.
            };
            if delta_rotation.abs() > FRAC_PI_2 {
                commands.entity(car).insert(AccelerateAction::Backward);
                steer_signum = -steer_signum;
            } else {
                commands.entity(car).insert(AccelerateAction::Forward);
            }
            let steering = steer_signum * aggression * (delta_rotation * FRAC_2_PI);
            commands.entity(car).insert(SteerAction(steering));
        }
    }

    #[cfg(feature = "gizmos")]
    fn render_bot_gizmos(
        mut gizmos: bevy::prelude::Gizmos,
        bots: Query<(&Transform, &LinearVelocity, &BotGoals)>,
    ) {
        use bevy::color::palettes::css::*;
        for (transform, velocity, goals) in &bots {
            let start = transform.translation.xy();
            for goal in &goals.0 {
                let bot_position = transform.translation.xy();
                gizmos.arrow_2d(
                    start,
                    start + goal.to_influence(bot_position, **velocity).target,
                    BLUE,
                );
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct BotControllerSystems;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
struct BotController;

#[derive(Bundle)]
pub struct BotControllerBundle {
    controller: BotController,
    entropy: Entropy,
    shapecast: ShapeCaster,
    goals: BotGoals,
}

impl BotControllerBundle {
    pub fn new(entropy: &mut GlobalEntropy) -> Self {
        Self {
            controller: BotController,
            entropy: entropy.fork_rng(),
            shapecast: ShapeCaster::new(CarPhysicsBundle::collider(), Vec2::ZERO, 0., Dir2::X)
                .with_max_hits(2)
                .with_max_time_of_impact(250.)
                .with_ignore_origin_penetration(true)
                .with_query_filter(SpatialQueryFilter::from_mask(
                    LayerMask::ALL & (!Checkpoint::COLLISION_LAYER),
                )),
            goals: BotGoals::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
#[derive(Component, Reflect)]
struct BotGoals(Vec<Goal>);

#[derive(Clone, Debug, PartialEq)]
#[derive(Reflect)]
enum Goal {
    MaxSpeed(Vec2),
    FollowTrack(Vec2),
    ReachCheckpoints(Vec<Vec2>),
    Chase { target: Vec2, priority: f32 },
    Avoid { target: Vec2, time_of_impact: f32 },
}

impl Goal {
    fn max_speed(bot_transform: Transform, bot_velocity: Vec2) -> Self {
        let facing_direction = if bot_velocity.length() < 0.001 {
            bot_transform.local_x().as_vec3().xy()
        } else {
            bot_velocity.normalize()
        };

        Goal::MaxSpeed(bot_transform.translation.xy() + facing_direction * 100.)
    }

    fn follow_track(bot_position: Vec2) -> Self {
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
            | TOP_LEFT_ANGLE..=EDGE_POS => bot_position.to_angle() + std::f32::consts::FRAC_PI_2,
            _ => panic!("Vec2::to_angle() returned outside [-pi, +pi]"),
        } + std::f32::consts::FRAC_PI_2;

        Goal::FollowTrack(bot_position + Vec2::from_angle(positional_ideal_rotation) * 100.)
    }

    fn reach_checkpoints<'a>(
        bot_position: Vec2,
        bot_velocity: Vec2,
        checkpoints: impl IntoIterator<Item = (Entity, &'a Checkpoint)> + 'a,
        tracker: &'a CheckpointTracker,
    ) -> Self {
        let mut checkpoints = checkpoints
            .into_iter()
            .filter(|(checkpoint, _)| !tracker.contains(checkpoint))
            .map(|(_, checkpoint)| {
                let origin = checkpoint.chunk.origin();
                let nearest_point = origin
                    + (bot_position - origin)
                        .project_onto(Vec2::from_angle(checkpoint.chunk.angle()));
                if nearest_point.distance_squared(checkpoint.position)
                    > Checkpoint::WIDTH * Checkpoint::WIDTH / 4.
                {
                    nearest_point
                } else {
                    checkpoint.position
                }
            })
            .collect::<Vec<_>>();

        let bot_predictive_position = bot_position + bot_velocity.normalize() * 100.;
        checkpoints.sort_by(|checkpoint1, checkpoint2| {
            let distance1 = checkpoint1.distance(bot_predictive_position);
            let distance2 = checkpoint2.distance(bot_predictive_position);
            distance1.total_cmp(&distance2)
        });
        Goal::ReachCheckpoints(checkpoints)
    }

    fn chase(target: Vec2, priority: f32) -> Self {
        Goal::Chase { target, priority }
    }

    fn avoid(target: Vec2, time_of_impact: f32) -> Self {
        Goal::Avoid {
            target,
            time_of_impact,
        }
    }

    fn to_influence(&self, bot_position: Vec2, bot_velocity: Vec2) -> Influence {
        match self {
            Goal::MaxSpeed(target) => Influence::new(*target, 4.),
            Goal::FollowTrack(target) => Influence::new(*target, 5.),
            Goal::ReachCheckpoints(targets) => {
                let (targets_ahead, targets_behind): (Vec<_>, Vec<_>) = targets
                    .iter()
                    .cloned()
                    .partition(|target| target.dot(bot_velocity).is_sign_positive());
                let filtered_targets = if targets_ahead.len() >= targets_behind.len() {
                    targets_ahead
                } else {
                    targets_behind
                };
                let farthest_checkpoint = filtered_targets
                    .iter()
                    .map(|target| target.distance(bot_position))
                    .max_by(|d1, d2| d1.total_cmp(d2))
                    .unwrap_or(f32::MAX);

                let average = weighted_avg(filtered_targets.into_iter().map(|target| {
                    Influence::new(
                        target,
                        (farthest_checkpoint - target.distance(bot_position)) / farthest_checkpoint,
                    )
                }));
                Influence::new(average, 10.)
            }
            Goal::Chase { target, priority } => Influence::new(*target, *priority),
            Goal::Avoid {
                target,
                time_of_impact,
            } => {
                if *time_of_impact < 100. {
                    Influence::new(-*target, 12.)
                } else {
                    let delta = *target - bot_position;
                    let prefer_left = delta.angle_between(bot_velocity).is_sign_positive();
                    let avoidance_offset = if prefer_left { 1. } else { -1. } * delta.perp();
                    Influence::new(*target + avoidance_offset * 30., 5.)
                }
            }
        }
    }
}

struct Influence {
    target: Vec2,
    strength: f32,
}

impl Influence {
    fn new(target: Vec2, strength: f32) -> Self {
        Self { target, strength }
    }
}

fn weighted_avg(collection: impl Iterator<Item = Influence>) -> Vec2 {
    let mut weight = 0.;
    let mut weighted_avg = Vec2::ZERO;
    let mut add_weight = |sample: Vec2, sample_weight: f32| {
        weight += sample_weight;
        weighted_avg = weighted_avg + (sample_weight / weight) * (sample - weighted_avg);
    };
    for Influence { target, strength } in collection {
        add_weight(target, strength);
    }
    weighted_avg
}
