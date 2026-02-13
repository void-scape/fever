use crate::{
    animation::AnimationSystems,
    fractal::{CPlane, Fractal, FractalMesh, JuliaCoordinates},
    state::GameState,
    transition::Transition,
};
use bevy::{prelude::*, render::view::Hdr};
use fever_macros::Lerp;

pub fn plugin(app: &mut App) {
    app.add_systems(OnExit(GameState::Loading), spawn)
        .add_systems(
            Update,
            move_camera
                .after(AnimationSystems::Interpolate)
                .run_if(in_state(GameState::Playing)),
        );
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MovementSensitivity::default(),
        Hdr,
        Transition::default(),
    ));
}

#[derive(Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct MovementSensitivity(pub f32);

impl Default for MovementSensitivity {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component)]
pub struct Stationary;

pub fn lock_camera(mut commands: Commands, camera: Single<Entity, With<Camera>>) {
    commands.entity(*camera).insert(Stationary);
}

pub fn unlock_camera(mut commands: Commands, camera: Single<Entity, With<Camera>>) {
    commands.entity(*camera).remove::<Stationary>();
}

#[derive(Component)]
pub struct ForceOrigin;

pub fn force_camera_origin(mut commands: Commands, camera: Single<Entity, With<Camera>>) {
    commands.entity(*camera).insert(ForceOrigin);
}

pub fn unforce_camera_origin(mut commands: Commands, camera: Single<Entity, With<Camera>>) {
    commands.entity(*camera).remove::<ForceOrigin>();
}

fn move_camera(
    transform: Single<
        (&mut Transform, Has<ForceOrigin>, Has<Stationary>),
        (With<Camera2d>, Without<FractalMesh>),
    >,
    cplane: Single<&CPlane, With<Fractal>>,
    coords: JuliaCoordinates,
) {
    let (mut transform, force_origin, stationary) = transform.into_inner();
    if force_origin {
        transform.translation = Vec3::ZERO;
        return;
    }
    if stationary {
        return;
    }
    let w = coords.world2(cplane.0);
    transform.translation.x = w.x;
    transform.translation.y = w.y;
}
