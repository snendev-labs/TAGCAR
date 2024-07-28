use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use car::{AccelerateAction, Car, DrivingSystems, SteerAction};
use entropy::{Entropy, RngCore};
use laptag::{BombTagIt, LapTagIt};

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
        mut commands: Commands,
        mut bots: Query<(Entity, Option<&mut BotState>), With<BotController>>,
        cars: Query<
            (
                &Transform,
                &LinearVelocity,
                Option<&LapTagIt>,
                Option<&BombTagIt>,
            ),
            With<Car>,
        >,
    ) {
        for (bot, state) in &mut bots {
            let Ok((bot_transform, bot_velocity, bot_lap_tag, bot_bomb_tag)) = cars.get(bot) else {
                continue;
            };
            let _bot_transform = *bot_transform;
            let _bot_velocity = **bot_velocity;
            let _has_lap_tag = bot_lap_tag.is_some();
            let _has_bomb_tag = bot_bomb_tag.is_some();

            // either do some raycasting or iterate cars and other things
            // avian2d::raycasting::

            let next_state = BotState {
                ideal_position: Vec2::ZERO,
                ideal_rotation: 0.,
            };
            if let Some(mut state) = state {
                *state = next_state;
            } else {
                commands.entity(bot).insert(next_state);
            }
        }
    }

    fn decide_bot_controls(
        mut commands: Commands,
        mut bots: Query<(Entity, &BotState, &mut Entropy), With<BotController>>,
    ) {
        for (car, _bot, mut entropy) in &mut bots {
            commands.entity(car).insert(AccelerateAction::Forward);

            // commands
            //     .entity(car_entity)
            //     .insert(AccelerateAction::Backward);
            // let _ = SteerAction(0.);
            let signum = if entropy.next_u32() < u32::MAX / 2 {
                1.
            } else {
                -1.
            };
            commands.entity(car).insert(SteerAction(signum * 0.1));
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct BotControlSystems;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct BotController;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
struct BotState {
    ideal_position: Vec2,
    ideal_rotation: f32,
}
