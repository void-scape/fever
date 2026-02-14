use crate::{
    animation::AnimationSystems,
    fractal::{CPlane, Fractal, FractalMesh, JuliaCoordinates},
    state::GameState,
};
use bevy::{
    color::palettes::css::RED,
    core_pipeline::{
        core_2d::graph::Node2d,
        fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
    },
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_graph::{InternedRenderLabel, RenderLabel},
        render_resource::ShaderType,
        view::Hdr,
    },
};
use fever_macros::Lerp;

pub fn camera_plugin(app: &mut App) {
    app.add_systems(OnExit(GameState::Loading), spawn)
        .add_systems(
            Update,
            move_camera
                .after(AnimationSystems::Interpolate)
                .run_if(in_state(GameState::Playing)),
        )
        .add_plugins(FullscreenMaterialPlugin::<CameraTransition>::default())
        .add_systems(Update, transition_progress);
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MovementSensitivity::default(),
        Hdr,
        CameraTransition::default(),
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

#[derive(Default, Clone, Copy, Lerp, Component, Deref, DerefMut)]
pub struct CameraTransitionProgress(pub f32);

fn transition_progress(
    time: Res<Time>,
    args: Single<
        (&mut CameraTransition, &CameraTransitionProgress),
        Changed<CameraTransitionProgress>,
    >,
) {
    let (mut transition, progress) = args.into_inner();
    transition.time += time.delta_secs();
    transition.progress = progress.0;
    transition.background_threshold = (1.0 - progress.0 * 2.0).abs() - 0.5;
    transition.color_threshold = (-4.0 + progress.0 * 8.0).abs().min(1.0) * 0.48;
}

#[derive(Clone, Copy, Component, ExtractComponent, ShaderType)]
#[require(CameraTransitionProgress)]
pub struct CameraTransition {
    color: LinearRgba,
    pixelation: Vec2,
    progress: f32,
    speed: f32,
    zoom: f32,
    background_threshold: f32,
    color_threshold: f32,
    seed: f32,
    time: f32,
}

impl Default for CameraTransition {
    fn default() -> Self {
        Self {
            pixelation: Vec2::splat(1.0),
            color: RED.into(),
            progress: 0.0,
            speed: 0.1,
            zoom: 2.0,
            background_threshold: 0.0,
            color_threshold: 0.0,
            seed: 420.0,
            time: 0.0,
        }
    }
}

impl FullscreenMaterial for CameraTransition {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/transition.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node2d::Tonemapping.intern(),
            Self::node_label().intern(),
            Node2d::EndMainPassPostProcessing.intern(),
        ]
    }
}
