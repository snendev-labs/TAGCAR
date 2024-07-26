use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use car::{AccelerateAction, Car, DrivingSystems, TurnAction};

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
        car_query: Query<Entity, (With<Car>, Without<InputMap<CarControl>>)>,
    ) {
        for car in &car_query {
            println!("Adding controller");
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
                println!("Accelerate pressed");
                commands
                    .entity(car_entity)
                    .insert(AccelerateAction::Forward);
            }

            if action_state.pressed(&CarControl::Brake) {
                println!("Deaccelerate pressed");
                commands
                    .entity(car_entity)
                    .insert(AccelerateAction::Backward);
            }

            let mut steering_angle: f32 = 0.;
            if action_state.pressed(&CarControl::TurnLeft) {
                println!("Turn left");
                steering_angle += Car::TURNING_ANGLE;
            }
            if action_state.pressed(&CarControl::TurnRight) {
                println!("Turn right");
                steering_angle -= Car::TURNING_ANGLE;
            }

            commands
                .entity(car_entity)
                .insert(TurnAction(steering_angle));
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct CarControlSystems;

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
}
