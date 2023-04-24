use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(input)
        .run();
}

#[derive(Component)]
struct Engine;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., -5.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands
        .spawn(Collider::cuboid(1000.0, 0.1, 1000.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -0.2, 0.0)));

    let body = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cylinder(0.5, 1.0),
            Restitution::coefficient(0.),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: 1.0,
                    height: 1.0,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.1, 0.1, 1.0).into()),
                transform: Transform::from_xyz(0., 0.5, 0.),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: 0.25,
                    height: 2.0,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.1, 0.1, 5.0).into()),
                transform: Transform::from_xyz(0., 1.0, 0.),
                ..default()
            });
        })
        .id();

    let engine_1 = commands
        .spawn((
            Engine,
            RigidBody::Dynamic,
            Collider::cylinder(0.5, 0.25),
            Restitution::coefficient(0.),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: 0.25,
                    height: 1.0,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.1, 0.1, 1.0).into()),
                transform: Transform::from_xyz(-2., 0.25, 0.),
                ..default()
            },
            ImpulseJoint::new(
                body,
                FixedJointBuilder::new()
                    .local_anchor1(Vec3::new(2., 0.25, 0.))
                    .local_anchor2(Vec3::ZERO),
            ),
            ExternalForce::default(),
        ))
        .id();

    let engine_2 = commands
        .spawn((
            Engine,
            RigidBody::Dynamic,
            Collider::cylinder(0.5, 0.25),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: 0.25,
                    height: 1.0,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.1, 0.1, 1.0).into()),
                transform: Transform::from_xyz(2., 0.25, 0.),
                ..default()
            },
            ImpulseJoint::new(
                body,
                FixedJointBuilder::new()
                    .local_anchor1(Vec3::new(-2., 0.25, 0.))
                    .local_anchor2(Vec3::ZERO),
            ),
            ExternalForce::default(),
        ))
        .id();

    commands.entity(body).push_children(&[engine_1, engine_2]);
}

fn input(keys: Res<Input<KeyCode>>, mut engine_query: Query<&mut ExternalForce, With<Engine>>) {
    if keys.pressed(KeyCode::Space) {
        for mut engine_force in engine_query.iter_mut() {
            engine_force.force = Vec3::Y * 20.0;
        }
    }
    if keys.just_released(KeyCode::Space) {
        for mut engine_force in engine_query.iter_mut() {
            engine_force.force = Vec3::ZERO;
        }
    }
}
