use std::time::Duration;

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct TriggerSlowmoSystems;

pub struct SlowmoPlugin;

impl Plugin for SlowmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::start_slowmo.in_set(TriggerSlowmoSystems))
            .add_systems(
                PreUpdate,
                Self::ease_in_clock.run_if(resource_exists::<SlowmoTimer>),
            );
    }
}

impl SlowmoPlugin {
    fn start_slowmo(
        mut commands: Commands,
        mut physics_time: ResMut<Time<Physics>>,
        mut slowmo_timer: Option<ResMut<SlowmoTimer>>,
    ) {
        const DURATION: Duration = Duration::from_secs(2);
        physics_time.set_relative_speed(0.25);
        if let Some(timer) = slowmo_timer.as_mut() {
            timer.0 = DURATION;
        } else {
            commands.insert_resource(SlowmoTimer(DURATION));
        }
    }

    fn ease_in_clock(
        mut commands: Commands,
        real_time: Res<Time<Real>>,
        mut physics_time: ResMut<Time<Physics>>,
        mut slowmo_timer: ResMut<SlowmoTimer>,
    ) {
        slowmo_timer.0 = slowmo_timer.0.saturating_sub(real_time.delta());
        let relative_speed =
            simple_easing::expo_in((Duration::from_secs(2) - slowmo_timer.0).as_secs_f32() / 2.0);
        physics_time.set_relative_speed(0.25 + 0.75 * relative_speed);
        if slowmo_timer.0.is_zero() {
            commands.remove_resource::<SlowmoTimer>();
        }
    }
}

#[derive(Resource)]
struct SlowmoTimer(Duration);
