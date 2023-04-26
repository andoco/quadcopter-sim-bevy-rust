use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(attach_to_follow);
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Follow;

fn setup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(MainCamera);
}

fn attach_to_follow(
    mut commands: Commands,
    follow_query: Query<Entity, Added<Follow>>,
    camera_query: Query<Entity, With<MainCamera>>,
) {
    let Ok(follow_entity) = follow_query.get_single() else {
        return;
    };
    let Ok(camera_entity) = camera_query.get_single() else {
        return;
    };

    commands.entity(camera_entity).set_parent(follow_entity);
    // .insert(Transform::from_xyz(0.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y));
}
