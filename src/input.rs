use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{Engine, Flyer, FlyerAction};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<FlyerAction>::default())
            .add_system(add_flyer_input)
            // .add_system(handle_flyer_input)
            .add_system(handle_quadcopter_flyer_input);
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
                    .insert(DualAxis::left_stick(), FlyerAction::Tilt)
                    .insert(DualAxis::right_stick(), FlyerAction::Lift)
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

    // if action_state.pressed(FlyerAction::Thrust2) {
    //     let axis_pair = action_state
    //         .clamped_axis_pair(FlyerAction::Thrust2)
    //         .unwrap();

    //     // force required to exceed gravity
    //     let min_force_required = -rapier_config.gravity.y * mass_props.mass;

    //     let thrust = axis_pair.y() * min_force_required * 2.0;

    //     thrust_force = Vec3::Y * thrust;
    // }

    let mut move_force = Vec3::ZERO;

    // if action_state.pressed(FlyerAction::Move) {
    //     let axis_pair = action_state.clamped_axis_pair(FlyerAction::Move).unwrap();

    //     move_force = -tx.forward() * axis_pair.y() * 5.0;
    //     spin_force = Vec3::Y * -axis_pair.x() * 5_f32.to_radians();
    // }

    ext_force.force = thrust_force + move_force;
    ext_force.torque = spin_force;
}

fn handle_quadcopter_flyer_input(
    mut query: Query<(&ActionState<FlyerAction>, &ReadMassProperties), With<Flyer>>,
    mut engine_query: Query<(
        &Engine,
        &Transform,
        &GlobalTransform,
        &ReadMassProperties,
        &mut ExternalForce,
    )>,
    rapier_config: Res<RapierConfiguration>,
) {
    let Some((action_state, ReadMassProperties(mass_props))) = query.get_single_mut().ok() else {
        return
    };

    // force required to exceed gravity
    let body_lift_force_required = -rapier_config.gravity.y * mass_props.mass;

    let mut engine_thrusts = vec![0.0; 4];

    for (Engine(engine_idx), _, _, ReadMassProperties(mass_props), _) in engine_query.iter_mut() {
        engine_thrusts[*engine_idx as usize] =
            body_lift_force_required / 4.0 + (-rapier_config.gravity.y * mass_props.mass);
    }

    let engine_torques = vec![1., -1., -1., 1.];

    if action_state.pressed(FlyerAction::Tilt) {
        let axis_pair = action_state.clamped_axis_pair(FlyerAction::Tilt).unwrap();

        let pitch_factor = 0.001;

        // Pitch forward
        let pitch_amount = axis_pair.y().max(0.0);
        engine_thrusts[0] -= pitch_amount * pitch_factor;
        engine_thrusts[1] -= pitch_amount * pitch_factor;
        engine_thrusts[2] += pitch_amount * pitch_factor;
        engine_thrusts[3] += pitch_amount * pitch_factor;

        // Yaw
        let yaw_factor = 0.01;
        engine_thrusts[0] += axis_pair.x() * yaw_factor;
        engine_thrusts[3] += axis_pair.x() * yaw_factor;
    }

    if action_state.pressed(FlyerAction::Lift) {
        let axis_pair = action_state.clamped_axis_pair(FlyerAction::Lift).unwrap();

        engine_thrusts = engine_thrusts
            .iter()
            .map(|f| f + f * axis_pair.y())
            .collect();
    }

    // info!("Applying thrust to engines: {:?}", engine_thrusts);

    for (Engine(engine_idx), _, global_tx, _, mut force) in engine_query.iter_mut() {
        force.force = global_tx.up() * engine_thrusts[*engine_idx as usize];
        force.torque = global_tx.up()
            * engine_thrusts[*engine_idx as usize]
            * engine_torques[*engine_idx as usize];
    }
}
