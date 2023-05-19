use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{Engine, Flyer, FlyerAction};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<FlyerAction>::default())
            .add_system(add_flyer_input)
            .add_system(update_required_engine_thrusts)
            .add_system(
                handle_keyboard_input
                    .after(update_required_engine_thrusts)
                    .before(apply_engine_thrusts),
            )
            .add_system(
                handle_gamepad_input
                    .after(update_required_engine_thrusts)
                    .before(apply_engine_thrusts),
            )
            .add_system(apply_engine_thrusts);
    }
}

fn add_flyer_input(mut commands: Commands, query: Query<Entity, Added<Flyer>>) {
    if let Some(entity) = query.get_single().ok() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::<FlyerAction> {
                action_state: ActionState::default(),
                input_map: InputMap::default()
                    .insert(KeyCode::Up, FlyerAction::Up)
                    .insert(KeyCode::Down, FlyerAction::Down)
                    .insert(KeyCode::Left, FlyerAction::Left)
                    .insert(KeyCode::Right, FlyerAction::Right)
                    .insert(KeyCode::A, FlyerAction::ThrustUp)
                    .insert(KeyCode::Z, FlyerAction::ThrustDown)
                    .insert(DualAxis::left_stick(), FlyerAction::Tilt)
                    .insert(DualAxis::right_stick(), FlyerAction::Lift)
                    .build(),
            });
    }
}

#[derive(Component)]
struct EngineThrusts(f32, f32, f32, f32);

impl EngineThrusts {
    fn to_array(&self) -> [f32; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

fn update_required_engine_thrusts(
    mut commands: Commands,
    mut query: Query<(Entity, &ReadMassProperties, &GlobalTransform), With<Flyer>>,
    engine_mass_query: Query<&ReadMassProperties, With<Engine>>,
    rapier_config: Res<RapierConfiguration>,
) {
    let Some((entity, ReadMassProperties(mass_props), global_tx)) = query.get_single_mut().ok() else {
        return
    };

    let pitch_angle = global_tx
        .compute_transform()
        .rotation
        .to_euler(EulerRot::XYZ)
        .0;

    let total_engine_mass = engine_mass_query
        .iter()
        .fold(0.0, |acc, ReadMassProperties(mass_props)| {
            acc + mass_props.mass
        });

    let total_mass = mass_props.mass + total_engine_mass;

    let gravity = rapier_config.gravity.y.abs();

    let min_total_thrust_required =
        calculate_thrust_required(gravity, pitch_angle, total_mass).max(0.0);

    info!(
        "total_mass = {}, pitch_angle = {}, thrust_required = {}",
        total_mass,
        pitch_angle.to_degrees(),
        min_total_thrust_required
    );

    let thrust_per_engine = min_total_thrust_required / 4.0;

    commands.entity(entity).insert(EngineThrusts(
        thrust_per_engine,
        thrust_per_engine,
        thrust_per_engine,
        thrust_per_engine,
    ));
}

fn handle_keyboard_input(
    mut query: Query<(&ActionState<FlyerAction>, &mut EngineThrusts), With<Flyer>>,
) {
    let Ok((action_state, mut engine_thrusts)) = query.get_single_mut() else {
        return
    };

    if action_state.pressed(FlyerAction::Left) {
        engine_thrusts.0 *= 1.1;
        engine_thrusts.3 *= 1.1;
    }
    if action_state.pressed(FlyerAction::Right) {
        engine_thrusts.1 *= 1.1;
        engine_thrusts.2 *= 1.1;
    }
    if action_state.pressed(FlyerAction::Up) {
        engine_thrusts.0 *= 0.9;
        engine_thrusts.1 *= 0.9;
        engine_thrusts.2 *= 1.1;
        engine_thrusts.3 *= 1.1;
    }
    if action_state.pressed(FlyerAction::Down) {
        engine_thrusts.0 *= 1.1;
        engine_thrusts.1 *= 1.1;
        engine_thrusts.2 *= 0.9;
        engine_thrusts.3 *= 0.9;
    }
    if action_state.pressed(FlyerAction::ThrustUp) {
        engine_thrusts.0 *= 2.0;
        engine_thrusts.1 *= 2.0;
        engine_thrusts.2 *= 2.0;
        engine_thrusts.3 *= 2.0;
    }
    if action_state.pressed(FlyerAction::ThrustDown) {
        engine_thrusts.0 *= -0.25;
        engine_thrusts.1 *= -0.25;
        engine_thrusts.2 *= -0.25;
        engine_thrusts.3 *= -0.25;
    }
}

fn handle_gamepad_input(
    mut flyer_query: Query<(&ActionState<FlyerAction>, &mut EngineThrusts), With<Flyer>>,
) {
    let Some((action_state,  mut engine_thrusts)) = flyer_query.get_single_mut().ok() else {
        return
    };

    if action_state.pressed(FlyerAction::Tilt) {
        let axis_pair = action_state.clamped_axis_pair(FlyerAction::Tilt).unwrap();

        // Pitch forward
        if axis_pair.y().abs() >= 0.25 {
            let pitch_factor = 0.001;

            let pitch_amount = axis_pair.y().max(0.0);
            engine_thrusts.0 -= pitch_amount * pitch_factor;
            engine_thrusts.1 -= pitch_amount * pitch_factor;
            engine_thrusts.2 += pitch_amount * pitch_factor;
            engine_thrusts.3 += pitch_amount * pitch_factor;
        }

        // Yaw
        if axis_pair.x().abs() >= 0.25 {
            let yaw_factor = 0.01;
            engine_thrusts.0 += axis_pair.x() * yaw_factor;
            engine_thrusts.3 += axis_pair.x() * yaw_factor;
        }
    }

    if action_state.pressed(FlyerAction::Lift) {
        let axis_pair = action_state.clamped_axis_pair(FlyerAction::Lift).unwrap();

        let f = axis_pair.y();

        engine_thrusts.0 += f;
        engine_thrusts.1 += f;
        engine_thrusts.2 += f;
        engine_thrusts.3 += f;
    }
}

fn apply_engine_thrusts(
    flyer_query: Query<&EngineThrusts, With<Flyer>>,
    mut engine_force_query: Query<(&Engine, &GlobalTransform, &mut ExternalForce)>,
) {
    let Ok(engine_thrusts) = flyer_query.get_single() else {
        return;
    };

    let engine_torques = [1., -1., -1., 1.];

    let engine_thrusts = engine_thrusts.to_array();

    info!("Applying thrust to engines: {:?}", engine_thrusts);

    for (Engine(engine_idx), global_tx, mut force) in engine_force_query.iter_mut() {
        let thrust = engine_thrusts[*engine_idx as usize];
        force.force = global_tx.up() * thrust; // BUG up() in wrong direction?
        force.torque = global_tx.up() * thrust * engine_torques[*engine_idx as usize];
    }
}

/// Calculate the thrust force required to offset the downward force due to gravity.
fn calculate_thrust_required(g: f32, tilt: f32, mass: f32) -> f32 {
    // Thrust = Weight * Sin(Pitch Angle)

    // Need to convert from bevy to std angle
    let a = 90_f32.to_radians() - tilt;

    match tilt {
        _ if tilt == 0.0 => mass * g,
        _ => mass * g * a.sin(),
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
    fn calculate_thrust_required_when_45_degrees() {
        assert_relative_eq!(
            calculate_thrust_required(G, 45_f32.to_radians(), MASS),
            693.671752344003121,
            max_relative = 0.0001
        );
    }
}
