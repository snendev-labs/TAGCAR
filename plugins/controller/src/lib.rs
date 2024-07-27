use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use car::{AccelerateAction, Car, DrivingSystems, SteerAction};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Reflect)]
#[derive(Actionlike)]
pub enum CarControl {
    Accelerate,
    Brake,
    TurnLeft,
    TurnRight,
}

pub struct CarControllerPlugin;

impl Plugin for CarControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CarControl>::default())
            .configure_sets(Update, CarControlSystems.before(DrivingSystems))
            .add_systems(
                Update,
                (Self::add_controller, Self::handle_controls)
                    .chain()
                    .in_set(CarControlSystems),
            );
    }
}

impl CarControllerPlugin {
    fn add_controller(
        mut commands: Commands,
        car_query: Query<(Entity, &Controller), Without<InputMap<CarControl>>>,
    ) {
        for (car, controller) in &car_query {
            println!("Adding controller");
            commands.entity(car).insert(InputManagerBundle::with_map(
                controller.leafwing_input_map(),
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
                steering_angle += 1.;
            }
            if action_state.pressed(&CarControl::TurnRight) {
                steering_angle -= 1.;
            }

            commands
                .entity(car_entity)
                .insert(SteerAction(steering_angle));
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct CarControlSystems;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub enum Controller {
    ArrowKeys,
    WASDKeys,
}

impl Controller {
    fn leafwing_input_map(&self) -> InputMap<CarControl> {
        match self {
            Controller::WASDKeys => InputMap::new([
                (CarControl::Accelerate, KeyCode::KeyW),
                (CarControl::Brake, KeyCode::KeyS),
                (CarControl::TurnLeft, KeyCode::KeyA),
                (CarControl::TurnRight, KeyCode::KeyD),
            ]),
            Controller::ArrowKeys => InputMap::new([
                (CarControl::Accelerate, KeyCode::ArrowUp),
                (CarControl::Brake, KeyCode::ArrowDown),
                (CarControl::TurnLeft, KeyCode::ArrowLeft),
                (CarControl::TurnRight, KeyCode::ArrowRight),
            ]),
        }
    }
}
