use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{Flyer, FlyerAction};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<FlyerAction>::default())
            .add_system(add_flyer_input)
            .add_system(handle_flyer_input);
    }
}

fn add_flyer_input(mut commands: Commands, query: Query<Entity, Added<Flyer>>) {
    if let Some(entity) = query.get_single().ok() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::<FlyerAction> {
                action_state: ActionState::default(),
                input_map: InputMap::new([
                    (KeyCode::Left, FlyerAction::Left),
                    (KeyCode::Right, FlyerAction::Right),
                    (KeyCode::Up, FlyerAction::Up),
                    (KeyCode::Down, FlyerAction::Down),
                    (KeyCode::Space, FlyerAction::Thrust),
                ]),
            });
    }
}

fn handle_flyer_input(
    mut query: Query<(&Transform, &mut ExternalForce, &ActionState<FlyerAction>), With<Flyer>>,
    time: Res<Time>,
) {
    let Some((tx, mut ext_force, action_state)) = query.get_single_mut().ok() else {
        return
    };

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
