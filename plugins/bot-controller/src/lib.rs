use avian2d::prelude::{
    LayerMask, LinearVelocity, Rotation, ShapeCaster, ShapeHits, SpatialQueryFilter,
};
use bevy::prelude::*;

use car::{AccelerateAction, CarPhysicsBundle, DrivingSystems, SteerAction};
use entropy::{Entropy, ForkableRng, GlobalEntropy, RngCore};
use laptag::{BombTagIt, CanBeIt, LapTagIt};
use resurfacer::Peg;
use track::{Checkpoint, CheckpointTracker, Track, Wall};

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
    #[allow(clippy::type_complexity)]
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
        track: Query<&Track>,
        players: Query<(Entity, &Transform, Option<&LapTagIt>, Option<&BombTagIt>), With<CanBeIt>>,
        obstacles: Query<&Transform, Or<(With<Wall>, With<Peg>)>>,
        _checkpoints: Query<(Entity, &Checkpoint)>,
    ) {
        let Ok(track) = track.get_single() else {
            return;
        };
        for (bot, bot_velocity, mut goals, shape_hits, _tracker) in &mut bots {
            let Ok((_, bot_transform, bot_lap_tag, bot_bomb_tag)) = players.get(bot) else {
                continue;
            };

            let bot_transform = *bot_transform;
            let bot_position = bot_transform.translation.xy();
            let _bot_has_lap_tag = bot_lap_tag.is_some();
            let _bot_has_bomb_tag = bot_bomb_tag.is_some();

            // establish some starting priorities for the bot
            let mut new_goals = BotGoals::default();
            new_goals
                .0
                .push(Goal::max_speed(bot_transform, **bot_velocity));
            new_goals
                .0
                .push(Goal::follow_track(bot_position, **bot_velocity, track));
            // if bot_has_lap_tag && !bot_has_bomb_tag {
            //     if let Some(tracker) = tracker {
            //         new_goals.0.push(Goal::reach_checkpoints(
            //             bot_position,
            //             **bot_velocity,
            //             checkpoints.iter(),
            //             tracker,
            //         ));
            //     }
            // }

            // now shapecast to check if we are trapped against something
            if let Some(hit) = shape_hits.iter().next() {
                if let Ok(transform) = players
                    .get(hit.entity)
                    .map(|(_, transform, _, _)| transform)
                    .or(obstacles.get(hit.entity))
                {
                    new_goals
                        .0
                        .push(Goal::avoid(transform.translation.xy(), hit.time_of_impact));
                }
            }

            *goals = new_goals;
        }
    }

    #[allow(clippy::type_complexity)]
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
        use std::f32::consts::{FRAC_PI_2, PI};
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
            let delta_rotation = modulo_radian(ideal_rotation - rotation.as_radians());

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

            let is_forward = delta_rotation.abs() > FRAC_PI_2;
            if is_forward {
                commands.entity(car).insert(AccelerateAction::Backward);
                steer_signum = -steer_signum;
            } else {
                commands.entity(car).insert(AccelerateAction::Forward);
            }
            let steering = steer_signum * aggression * delta_rotation;
            commands.entity(car).insert(SteerAction(steering));
        }
    }

    #[cfg(feature = "gizmos")]
    fn render_bot_gizmos(
        mut gizmos: bevy::prelude::Gizmos,
        bots: Query<(&Transform, &LinearVelocity, &Rotation, &BotGoals)>,
    ) {
        use bevy::color::palettes::css::*;
        for (transform, velocity, rotation, goals) in &bots {
            let bot_position = transform.translation.xy();
            gizmos.arrow_2d(
                bot_position,
                bot_position + Vec2::from_angle(rotation.as_radians()) * 50.,
                ORANGE,
            );
            for goal in &goals.0 {
                gizmos.arrow_2d(
                    bot_position,
                    goal.to_influence(bot_position, **velocity).target,
                    match goal {
                        Goal::FollowTrack(_) => PURPLE,
                        Goal::MaxSpeed(_) => BLUE,
                        _ => PINK,
                    },
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
    Avoid { target: Vec2, time_of_impact: f32 },
    // ReachCheckpoints(Vec<Vec2>),
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

    fn follow_track(bot_position: Vec2, bot_velocity: Vec2, track: &Track) -> Self {
        let straightaway_max = Vec2::new(
            (track.half_length() - track.radius()) * 0.8,
            track.radius() * 1.1,
        );
        let straightaway_min = Vec2::new(
            (track.half_length() - track.radius()) * 0.8,
            track.interior_radius(),
        );
        let top_straightaway_bounds =
            Rect::from_corners(straightaway_min * Vec2::new(-1., 1.), straightaway_max);
        let bottom_straightaway_bounds =
            Rect::from_corners(-straightaway_max, straightaway_min * Vec2::new(1., -1.));
        let interior_max = Vec2::new(
            track.half_length() + track.radius(),
            track.interior_radius(),
        );
        // it's ok to treat the rect corners as part of the interior
        let interior_bounds = Rect::from_corners(-interior_max, interior_max);

        fn less(vec1: Vec2, vec2: Vec2) -> bool {
            vec1.x < vec2.x && vec1.y < vec2.y
        }

        let desired_direction =
            if less(interior_bounds.min, bot_position) && less(bot_position, interior_bounds.max) {
                // if you're in the middle, just keep moving
                bot_velocity.normalize()
            } else if less(top_straightaway_bounds.min, bot_position)
                && less(bot_position, top_straightaway_bounds.max)
            {
                // top goes left
                Vec2::NEG_X
            } else if less(bottom_straightaway_bounds.min, bot_position)
                && less(bot_position, bottom_straightaway_bounds.max)
            {
                // bottom goes right
                Vec2::X
            } else if bot_position.x.is_sign_positive() && bot_position.y.is_sign_positive() {
                // top right goes left
                Vec2::NEG_X
            } else if bot_position.x.is_sign_positive() && bot_position.y.is_sign_negative() {
                // bottom right goes up
                Vec2::Y
            } else if bot_position.x.is_sign_negative() && bot_position.y.is_sign_negative() {
                // bottom left goes right
                Vec2::X
            } else if bot_position.x.is_sign_negative() && bot_position.y.is_sign_positive() {
                // top left goes down
                Vec2::NEG_Y
            } else {
                // everyone else goes left
                Vec2::NEG_X
            };

        Goal::FollowTrack(bot_position + desired_direction * 100.)
    }

    fn avoid(target: Vec2, time_of_impact: f32) -> Self {
        Goal::Avoid {
            target,
            time_of_impact,
        }
    }

    // fn reach_checkpoints<'a>(
    //     bot_position: Vec2,
    //     bot_velocity: Vec2,
    //     checkpoints: impl IntoIterator<Item = (Entity, &'a Checkpoint)> + 'a,
    //     tracker: &'a CheckpointTracker,
    // ) -> Self {
    //     let mut checkpoints = checkpoints
    //         .into_iter()
    //         .filter(|(checkpoint, _)| !tracker.contains(checkpoint))
    //         .map(|(_, checkpoint)| {
    //             let origin = checkpoint.chunk.origin();
    //             let nearest_point = origin
    //                 + (bot_position - origin)
    //                     .project_onto(Vec2::from_angle(checkpoint.chunk.angle()));
    //             if nearest_point.distance_squared(checkpoint.position)
    //                 > Checkpoint::WIDTH * Checkpoint::WIDTH / 4.
    //             {
    //                 nearest_point
    //             } else {
    //                 checkpoint.position
    //             }
    //         })
    //         .collect::<Vec<_>>();

    //     let bot_predictive_position = bot_position + bot_velocity.normalize() * 100.;
    //     checkpoints.sort_by(|checkpoint1, checkpoint2| {
    //         let distance1 = checkpoint1.distance(bot_predictive_position);
    //         let distance2 = checkpoint2.distance(bot_predictive_position);
    //         distance1.total_cmp(&distance2)
    //     });
    //     Goal::ReachCheckpoints(checkpoints)
    // }

    fn to_influence(&self, bot_position: Vec2, bot_velocity: Vec2) -> Influence {
        match self {
            Goal::MaxSpeed(target) => Influence::new(*target, 1.),
            Goal::FollowTrack(target) => Influence::new(*target, 5.),
            Goal::Avoid {
                target,
                time_of_impact,
            } => {
                if *time_of_impact < 50. {
                    Influence::new(bot_position - (*target - bot_position), 100.)
                } else {
                    let delta = *target - bot_position;
                    let prefer_left = delta.angle_between(bot_velocity).is_sign_positive();
                    let avoidance_offset = if prefer_left { 1. } else { -1. } * delta.perp();
                    Influence::new(*target + avoidance_offset * 30., 5.)
                }
            } // Goal::ReachCheckpoints(targets) => {
              //     let (targets_ahead, targets_behind): (Vec<_>, Vec<_>) = targets
              //         .iter()
              //         .cloned()
              //         .partition(|target| target.dot(bot_velocity).is_sign_positive());
              //     let filtered_targets = if targets_ahead.len() >= targets_behind.len() {
              //         targets_ahead
              //     } else {
              //         targets_behind
              //     };
              //     let farthest_checkpoint = filtered_targets
              //         .iter()
              //         .map(|target| target.distance(bot_position))
              //         .max_by(|d1, d2| d1.total_cmp(d2))
              //         .unwrap_or(f32::MAX);

              //     let average = weighted_avg(filtered_targets.into_iter().map(|target| {
              //         Influence::new(
              //             target,
              //             (farthest_checkpoint - target.distance(bot_position)) / farthest_checkpoint,
              //         )
              //     }));
              //     Influence::new(average, 10.)
              // }
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

// code adapted from https://github.com/tguichaoua/angulus/blob/main/src/angle.rs#L160
fn modulo_radian(angle_radians: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    debug_assert!(angle_radians.is_nan() || (-TAU..=TAU).contains(&angle_radians));
    let angle_radians = angle_radians % TAU;
    if angle_radians > PI {
        angle_radians - TAU
    } else if angle_radians <= -PI {
        angle_radians + TAU
    } else {
        angle_radians
    }
}
