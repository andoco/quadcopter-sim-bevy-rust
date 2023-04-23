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
                input_map: InputMap::default()
                    .insert(KeyCode::Space, FlyerAction::Thrust)
                    .insert(DualAxis::left_stick(), FlyerAction::Move)
                    .insert(DualAxis::right_stick(), FlyerAction::Thrust2)
                    .build(),
            });
    }
}

fn handle_flyer_input(
    mut query: Query<
        (
            &Transform,
            &mut ExternalForce,
            &ReadMassProperties,
            &ActionState<FlyerAction>,
        ),
        With<Flyer>,
    >,
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
) {
    let Some((tx, mut ext_force, ReadMassProperties(mass_props), action_state)) = query.get_single_mut().ok() else {
        return
    };

    // ext_force.force *= 1.0 - time.delta_seconds();
    // ext_force.torque *= 1.0 - time.delta_seconds();

    // if ext_force.force.length() < 0.1 {
    //     ext_force.force = Vec3::ZERO;
    // }
    // if ext_force.torque.length() < 0.1 {
    //     ext_force.torque = Vec3::ZERO;
    // }

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

    let mut thrust_force = Vec3::ZERO;
    let mut spin_force = Vec3::ZERO;

    if action_state.pressed(FlyerAction::Thrust2) {
        let axis_pair = action_state
            .clamped_axis_pair(FlyerAction::Thrust2)
            .unwrap();

        // force required to exceed gravity
        let min_force_required = -rapier_config.gravity.y * mass_props.mass;

        let thrust = axis_pair.y() * min_force_required * 2.0;

        thrust_force = Vec3::Y * thrust;
    }

    let mut move_force = Vec3::ZERO;

    if action_state.pressed(FlyerAction::Move) {
        let axis_pair = action_state.clamped_axis_pair(FlyerAction::Move).unwrap();

        move_force = -tx.forward() * axis_pair.y() * 5.0;
        spin_force = Vec3::Y * -axis_pair.x() * 5_f32.to_radians();
    }

    ext_force.force = thrust_force + move_force;
    ext_force.torque = spin_force;
}
