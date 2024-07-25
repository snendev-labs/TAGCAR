use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use car::{AccelerateAction, Car, TurnAction};

pub struct CarControllerPlugin;

impl Plugin for CarControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CarControl>::default())
            .add_systems(Startup, CarControl::add_controller)
            .add_systems(Update, CarControl::handle_controls);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Reflect)]
#[derive(Actionlike)]
pub enum CarControl {
    Accelerate,
    Brake,
    TurnLeft,
    TurnRight,
}

impl CarControl {
    fn leafwing_input_map() -> InputMap<CarControl> {
        let mut input_map = InputMap::default();

        input_map.insert(CarControl::Accelerate, KeyCode::ArrowUp);
        input_map.insert(CarControl::Brake, KeyCode::ArrowDown);
        input_map.insert(CarControl::TurnLeft, KeyCode::ArrowLeft);
        input_map.insert(CarControl::TurnRight, KeyCode::ArrowRight);

        input_map
    }

    fn add_controller(mut commands: Commands, car_query: Query<Entity, With<Car>>) {
        for car in &car_query {
            commands.entity(car).insert(InputManagerBundle::with_map(
                CarControl::leafwing_input_map(),
            ));
        }
    }

    fn handle_controls(
        mut commands: Commands,
        car_query: Query<(Entity, &ActionState<CarControl>), With<Car>>,
    ) {
        for (car_entity, action_state) in &car_query {
            if action_state.pressed(&CarControl::Accelerate) {
                commands
                    .entity(car_entity)
                    .insert(AccelerateAction::Forward);
            }

            if action_state.pressed(&CarControl::Brake) {
                commands
                    .entity(car_entity)
                    .insert(AccelerateAction::Backward);
            }

            let mut steering_angle: f32 = 0.;
            if action_state.pressed(&CarControl::TurnLeft) {
                steering_angle += Car::TURNING_ANGLE;
            }
            if action_state.pressed(&CarControl::TurnRight) {
                steering_angle -= Car::TURNING_ANGLE;
            }

            commands
                .entity(car_entity)
                .insert(TurnAction(steering_angle));
        }
    }
}
