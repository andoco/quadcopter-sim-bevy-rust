use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{Engine, Flyer, FlyerAction};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<FlyerAction>::default())
            .add_system(add_flyer_input)
            .add_system(handle_keyboard_input)
            .add_system(handle_gamepad_input);
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

fn handle_keyboard_input(
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

fn handle_gamepad_input(
    mut query: Query<
        (
            &ActionState<FlyerAction>,
            &ReadMassProperties,
            &GlobalTransform,
        ),
        With<Flyer>,
    >,
    engine_mass_query: Query<&ReadMassProperties, With<Engine>>,
    mut engine_force_query: Query<(&Engine, &GlobalTransform, &mut ExternalForce)>,
    rapier_config: Res<RapierConfiguration>,
) {
    let Some((action_state, ReadMassProperties(mass_props), global_tx)) = query.get_single_mut().ok() else {
        return
    };

    let (tilt_angle_x, _, tilt_angle_z) = global_tx
        .compute_transform()
        .rotation
        .to_euler(EulerRot::XYZ);

    let total_engine_mass = engine_mass_query
        .iter()
        .fold(0.0, |acc, ReadMassProperties(mass_props)| {
            acc + mass_props.mass
        });

    let total_mass = mass_props.mass + total_engine_mass;

    let min_total_thrust_required =
        calculate_thrust_required(rapier_config.gravity.y.abs(), tilt_angle_x, total_mass).max(0.0);

    info!(
        "total_mass = {}, tilt_angle_x = {}, tilt_angle_z = {}, thrust_required = {}",
        total_mass,
        tilt_angle_x.to_degrees(),
        tilt_angle_z.to_degrees(),
        min_total_thrust_required
    );

    let engine_torques = vec![1., -1., -1., 1.];
    let mut engine_thrusts = vec![min_total_thrust_required / 4.0; 4];

    if action_state.pressed(FlyerAction::Tilt) {
        let axis_pair = action_state.clamped_axis_pair(FlyerAction::Tilt).unwrap();

        // Pitch forward
        if axis_pair.y().abs() >= 0.25 {
            let pitch_factor = 0.001;

            let pitch_amount = axis_pair.y().max(0.0);
            engine_thrusts[0] -= pitch_amount * pitch_factor;
            engine_thrusts[1] -= pitch_amount * pitch_factor;
            engine_thrusts[2] += pitch_amount * pitch_factor;
            engine_thrusts[3] += pitch_amount * pitch_factor;
        }

        // Yaw
        if axis_pair.x().abs() >= 0.25 {
            let yaw_factor = 0.01;
            engine_thrusts[0] += axis_pair.x() * yaw_factor;
            engine_thrusts[3] += axis_pair.x() * yaw_factor;
        }
    }

    if action_state.pressed(FlyerAction::Lift) {
        let axis_pair = action_state.clamped_axis_pair(FlyerAction::Lift).unwrap();

        engine_thrusts = engine_thrusts
            .iter()
            .map(|f| f + f * axis_pair.y())
            .collect();
    }

    info!("Applying thrust to engines: {:?}", engine_thrusts);

    for (Engine(engine_idx), global_tx, mut force) in engine_force_query.iter_mut() {
        let thrust = engine_thrusts[*engine_idx as usize];
        let thrust_2 = thrust + tilt_angle_x.sin() * thrust;
        force.force = global_tx.up() * thrust_2;
        force.torque = global_tx.up() * thrust_2 * engine_torques[*engine_idx as usize];
    }
}

/// Calculate the thrust force required to offset the downward force due to gravity.
fn calculate_thrust_required(g: f32, tilt: f32, mass: f32) -> f32 {
    // f * sin(tilt) = g * mass;
    // f = (g * mass) / sin(tilt)

    let a = 90_f32.to_radians() - tilt;

    match tilt {
        _ if tilt == 0.0 => g * mass,
        _ => (g * mass) / a.sin(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;

    const G: f32 = 9.81;
    const MASS: f32 = 100.0;

    #[test]
    fn calculate_thrust_required_when_0_degrees() {
        assert_eq!(
            calculate_thrust_required(G, 0_f32.to_radians(), MASS),
            G * MASS
        );
    }

    #[test]
    fn calculate_thrust_required_when_1_degrees() {
        assert_relative_eq!(
            calculate_thrust_required(G, 1_f32.to_radians(), MASS),
            981.1494,
            max_relative = 0.0001
        );
    }

    #[test]
    fn calculate_thrust_required_when_45_degrees() {
        assert_eq!(
            calculate_thrust_required(G, 45_f32.to_radians(), MASS),
            1387.3436
        );
    }

    #[test]
    fn calculate_thrust_required_when_89_degrees() {
        assert_relative_eq!(
            calculate_thrust_required(G, 89_f32.to_radians(), MASS),
            56210.0134,
            max_relative = 0.0001
        );
    }
}
