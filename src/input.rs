use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{Flyer, FlyerAction};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<FlyerAction>::default())
            .add_system(turn);
    }
}

fn turn(
    mut query: Query<(&Transform, &mut ExternalForce, &ActionState<FlyerAction>), With<Flyer>>,
    time: Res<Time>,
) {
    let (tx, mut ext_force, action_state) = query.single_mut();

    ext_force.force *= 1.0 - time.delta_seconds();
    ext_force.torque *= 1.0 - time.delta_seconds();

    if ext_force.force.length() < 0.1 {
        ext_force.force = Vec3::ZERO;
    }
    if ext_force.torque.length() < 0.1 {
        ext_force.torque = Vec3::ZERO;
    }

    let turn_rate = 15_f32.to_radians();
    let accel_rate = 10_f32;
    let thrust_rate = 10_f32;

    if action_state.pressed(FlyerAction::Left) {
        ext_force.torque += Vec3::Y * turn_rate * time.delta_seconds();
    }
    if action_state.pressed(FlyerAction::Right) {
        ext_force.torque += Vec3::Y * -turn_rate * time.delta_seconds();
    }
    if action_state.pressed(FlyerAction::Up) {
        ext_force.force += -tx.forward() * accel_rate * time.delta_seconds();
    }
    if action_state.pressed(FlyerAction::Down) {
        ext_force.force += tx.forward() * accel_rate * time.delta_seconds();
    }
    if action_state.pressed(FlyerAction::Thrust) {
        ext_force.force += tx.up() * thrust_rate * time.delta_seconds();
    }
}
