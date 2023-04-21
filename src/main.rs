use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_startup_system(setup_buildings)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
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
        .spawn(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    /* Create the bouncing ball. */

    let mut rng = rand::thread_rng();

    for _ in 0..20 {
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
