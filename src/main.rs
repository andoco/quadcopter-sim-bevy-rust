mod camera;
mod input;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use camera::CameraPlugin;
use input::InputPlugin;
use leafwing_input_manager::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(CameraPlugin)
        .add_plugin(InputPlugin)
        .add_startup_system(setup_physics)
        .add_startup_system(setup_buildings)
        .add_startup_system(setup_flyer)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum FlyerAction {
    Left,
    Right,
    Up,
    Down,
    Thrust,
    Tilt,
    Lift,
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
                    transform: Transform::from_translation(pos).with_scale(Vec3::new(5., 0.1, 5.)),
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
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 2000.,
                ..default()
            })),
            material: materials.add(Color::rgb(0.1, 0.1, 0.1).into()),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -0.2, 0.0)));

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

#[derive(Component, Clone)]
struct Engine(u8);

fn setup_flyer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let radius = 0.5;
    let height = 0.25;

    let engine_radius = radius * 0.25;
    let engine_height = height;
    let engine_offset = radius + (engine_radius * 2.0);

    let engine_colors = vec![Color::RED, Color::GREEN, Color::BLUE, Color::WHITE];

    let engine_bundle = (
        RigidBody::Dynamic,
        Collider::cylinder(engine_height / 2.0, engine_radius),
        Restitution::coefficient(0.),
        ExternalForce::default(),
        ReadMassProperties::default(),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: engine_radius,
                height: engine_height,
                ..default()
            })),
            material: materials.add(Color::rgb(0.1, 0.1, 1.0).into()),
            ..default()
        },
    );

    let body = commands
        .spawn((
            Flyer,
            RigidBody::Dynamic,
            Collider::cylinder(height / 2.0, radius),
            Restitution::coefficient(0.),
            ReadMassProperties::default(),
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
            camera::Follow,
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: radius * 0.25,
                    height: height * 0.25,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.8, 0.1, 1.0).into()),
                transform: Transform::from_xyz(0., height * 0.75, 0.),
                ..default()
            });
        })
        .id();

    let offset = engine_offset;
    let offsets = vec![
        Vec3::new(-offset, 0., offset),
        Vec3::new(offset, 0., offset),
        Vec3::new(-offset, 0., -offset),
        Vec3::new(offset, 0., -offset),
    ];

    let mut engines = vec![];

    for (i, offset) in offsets.iter().enumerate() {
        let joint = FixedJointBuilder::new()
            .local_anchor1(*offset)
            .local_anchor2(Vec3::ZERO);

        let color = engine_colors[i];

        let engine = commands
            .spawn(engine_bundle.clone())
            .insert(materials.add(color.into()))
            .insert(Engine(i as u8))
            .insert(TransformBundle::from(Transform::from_translation(*offset)))
            .insert(ImpulseJoint::new(body, joint))
            .id();

        engines.push(engine);
    }

    commands.entity(body).push_children(&engines);
}
