use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use car::{AccelerateAction, BrakeAction, Car, TurnAction};

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum DriveInput {
    Accelerate,
    Brake,
    TurnLeft,
    TurnRight,
}

impl DriveInput {
    fn car_input_map() -> InputMap<DriveInput> {
        let mut input_map = InputMap::default();

        input_map.insert(DriveInput::Accelerate, KeyCode::ArrowUp);
        input_map.insert(DriveInput::Brake, KeyCode::ArrowDown);
        input_map.insert(DriveInput::TurnLeft, KeyCode::ArrowLeft);
        input_map.insert(DriveInput::TurnRight, KeyCode::ArrowRight);

        input_map
    }

    fn car_input(
        mut commands: Commands,
        car_query: Query<(Entity, &ActionState<DriveInput>), With<Car>>,
    ) {
        for (car_entity, action_state) in &car_query {
            if action_state.pressed(DriveInput::Accelerate) {
                commands.entity(car_entity).insert(AccelerateAction);
            }

            if action_state.pressed(DriveInput::Brake) {
                commands.entity(car_entity).insert(BrakeAction);
            }

            let mut steering_angle: f32 = 0.;
            if action_state.pressed(DriveInput::TurnLeft) {
                steering_angle += Car::TURNING_ANGLE;
            }
            if action_state.pressed(DriveInput::TurnRight) {
                steering_angle -= Car::TURNING_ANGLE;
            }

            commands
                .entity(car_entity)
                .insert(TurnAction(steering_angle))
        }
    }
}
