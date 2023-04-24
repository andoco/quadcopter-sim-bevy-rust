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
        transform: Transform::from_xyz(0., 0., -10.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands
        .spawn(Collider::cuboid(1000.0, 0.1, 1000.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -0.2, 0.0)));

    let radius = 1.0;
    let height = 1.0;

    let body = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cylinder(height / 2.0, radius),
            Restitution::coefficient(0.),
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius,
                    height,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.1, 0.1, 1.0).into()),
                transform: Transform::from_xyz(0., height / 2.0, 0.),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: radius * 0.25,
                    height: height * 2.0,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.1, 0.1, 5.0).into()),
                transform: Transform::from_xyz(0., height, 0.),
                ..default()
            });
        })
        .id();

    let engine_radius = radius * 0.25;
    let engine_height = height;
    let offset_amount = radius + radius * 2.0;

    let offsets = vec![
        Vec3::new(-offset_amount, 0., offset_amount),
        Vec3::new(offset_amount, 0., offset_amount),
        Vec3::new(-offset_amount, 0., -offset_amount),
        Vec3::new(offset_amount, 0., -offset_amount),
    ];

    let mut engines = vec![];

    for offset in offsets {
        let engine = commands
            .spawn((
                Engine,
                RigidBody::Dynamic,
                Collider::cylinder(engine_height / 2.0, engine_radius),
                Restitution::coefficient(0.),
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cylinder {
                        radius: engine_radius,
                        height: engine_height,
                        ..default()
                    })),
                    material: materials.add(Color::rgb(0.1, 0.1, 1.0).into()),
                    transform: Transform::from_translation(offset),
                    ..default()
                },
                ImpulseJoint::new(
                    body,
                    FixedJointBuilder::new()
                        .local_anchor1(offset)
                        .local_anchor2(Vec3::ZERO),
                ),
                ExternalForce::default(),
            ))
            .id();

        engines.push(engine);
    }

    commands.entity(body).push_children(&engines);
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
