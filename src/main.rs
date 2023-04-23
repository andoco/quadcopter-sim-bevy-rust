use bevy::prelude::*;
use bevy_dolly::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_startup_system(setup_buildings)
        .add_startup_system(setup_flyer)
        .add_system(update_camera)
        .add_system(Dolly::<MainCamera>::update_active)
        .add_system(turn)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Left,
    Right,
    Up,
    Down,
    Thrust,
}

#[derive(Component)]
struct MainCamera;

fn setup_graphics(mut commands: Commands) {
    let start_pos = Vec3::new(0., 0., 0.);

    commands.spawn((
        MainCamera,
        Rig::builder()
            .with(MovableLookAt::from_position_target(start_pos))
            .build(),
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 20.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
    ));
}

fn update_camera(q0: Query<(&Transform, With<Flyer>)>, mut q1: Query<&mut Rig>) {
    let player = q0.single().0.to_owned();
    let mut rig = q1.single_mut();

    rig.driver_mut::<MovableLookAt>()
        .set_position_target(player.translation, player.rotation);
}

fn setup_buildings(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for row in -5..5 {
        for col in -5..5 {
            let pos = Vec3::new(col as f32 * 10.0, 0.0, row as f32 * 10.0);

            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_translation(pos).with_scale(Vec3::new(5., 5., 5.)),
                    ..default()
                })
                .insert(RigidBody::Fixed)
                .insert(Collider::cuboid(0.5, 0.5, 0.5));
        }
    }
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the ground. */
    commands
        .spawn(Collider::cuboid(1000.0, 0.1, 1000.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -0.1, 0.0)));

    let mut rng = rand::thread_rng();

    for _ in 0..5 {
        let pos = Vec3::new(
            rng.gen_range(-50..=50) as f32,
            50.,
            rng.gen_range(-50..=50) as f32,
        );

        let vel = Vec3::new(
            rng.gen_range(-10..=10) as f32,
            0.,
            rng.gen_range(-10..=10) as f32,
        );

        commands
            .spawn(RigidBody::Dynamic)
            .insert(Collider::ball(0.5))
            .insert(Restitution::coefficient(0.95))
            .insert(Velocity {
                linvel: vel,
                angvel: Vec3::new(0.0, 0.0, 0.0),
            })
            .insert(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: 0.5,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
                // transform: Transform::from_translation(pos).with_scale(Vec3::new(5., 5., 5.)),
                ..default()
            })
            .insert(TransformBundle::from(Transform::from_translation(pos)));
    }
}

#[derive(Component)]
struct Flyer;

fn setup_flyer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((RigidBody::Dynamic, Collider::cylinder(0.1, 0.5)))
        .insert(Restitution::coefficient(0.))
        .insert(Damping {
            linear_damping: 0.5,
            angular_damping: 1.0,
        })
        .insert(ExternalForce::default())
        // .insert(GravityScale(0.))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Torus {
                radius: 0.4,
                ring_radius: 0.1,
                ..default()
            })),
            material: materials.add(Color::rgb(0.1, 0.1, 1.0).into()),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0., 10., -100.)))
        .insert(Flyer)
        .insert(InputManagerBundle::<Action> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new([
                (KeyCode::Left, Action::Left),
                (KeyCode::Right, Action::Right),
                (KeyCode::Up, Action::Up),
                (KeyCode::Down, Action::Down),
                (KeyCode::Space, Action::Thrust),
            ]),
        });
}

fn turn(
    mut query: Query<(&Transform, &mut ExternalForce, &ActionState<Action>), With<Flyer>>,
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

    if action_state.pressed(Action::Left) {
        ext_force.torque += Vec3::Y * turn_rate * time.delta_seconds();
    }
    if action_state.pressed(Action::Right) {
        ext_force.torque += Vec3::Y * -turn_rate * time.delta_seconds();
    }
    if action_state.pressed(Action::Up) {
        ext_force.force += -tx.forward() * accel_rate * time.delta_seconds();
    }
    if action_state.pressed(Action::Down) {
        ext_force.force += tx.forward() * accel_rate * time.delta_seconds();
    }
    if action_state.pressed(Action::Thrust) {
        ext_force.force += tx.up() * thrust_rate * time.delta_seconds();
    }
}
