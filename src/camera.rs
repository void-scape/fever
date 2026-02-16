use crate::prelude::*;
use bevy::{
    color::palettes::{
        css::BLACK,
        tailwind::{BLUE_950, RED_950, ROSE_800, VIOLET_800},
    },
    core_pipeline::{
        core_2d::graph::Node2d,
        fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
    },
    post_process::effect_stack::ChromaticAberration,
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
    app.init_resource::<TransitionPalette>()
        .add_observer(reset_camera)
        .add_systems(OnExit(GameState::Loading), spawn)
        .add_systems(Update, move_camera.after(AnimationSystems::Interpolate))
        .add_plugins(FullscreenMaterialPlugin::<CameraTransition>::default())
        .add_systems(
            Update,
            (transition_progress, transition_time, transition_palette),
        );
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MovementSensitivity::default(),
        Hdr,
        CameraTransition::default(),
    ));
}

#[derive(Component)]
pub struct ResetCamera;

fn reset_camera(camera: On<Insert, ResetCamera>, mut commands: Commands) {
    commands
        .entity(camera.entity)
        .insert((
            MovementSensitivity::default(),
            CameraTransitionProgress(0.0),
        ))
        .remove::<(
            ChromaticAberration,
            AberrationIntensity,
            ForceOrigin,
            Stationary,
        )>();
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
    args: Single<
        (&mut CameraTransition, &CameraTransitionProgress),
        Changed<CameraTransitionProgress>,
    >,
) {
    let (mut transition, progress) = args.into_inner();
    transition.progress = progress.0;
    transition.background_threshold = (1.0 - progress.0 * 2.0).abs() - 0.5;
    transition.color_low_threshold = (-4.0 + progress.0 * 8.0).abs().min(1.0) * 0.24;
    transition.color_mid_threshold = (-4.0 + progress.0 * 8.0).abs().min(1.0) * 0.48;
}

fn transition_time(time: Res<Time>, mut args: Single<&mut CameraTransition>) {
    args.time = time.elapsed_secs_wrapped();
}

#[derive(Default, Resource)]
pub enum TransitionPalette {
    #[default]
    Blue,
    Red,
}

fn transition_palette(mut args: Single<&mut CameraTransition>, palette: Res<TransitionPalette>) {
    if palette.is_changed() {
        match *palette {
            TransitionPalette::Blue => {
                args.color_low = BLACK.into();
                args.color_mid = BLUE_950.into();
                args.color_high = VIOLET_800.into();
            }
            TransitionPalette::Red => {
                args.color_low = BLACK.into();
                args.color_mid = RED_950.into();
                args.color_high = ROSE_800.into();
            }
        }
    }
}

#[derive(Clone, Copy, Component, ExtractComponent, ShaderType)]
#[require(CameraTransitionProgress)]
pub struct CameraTransition {
    color_low: LinearRgba,
    color_mid: LinearRgba,
    color_high: LinearRgba,
    progress: f32,
    speed: f32,
    zoom: f32,
    background_threshold: f32,
    color_low_threshold: f32,
    color_mid_threshold: f32,
    seed: f32,
    time: f32,
}

impl Default for CameraTransition {
    fn default() -> Self {
        Self {
            color_low: BLACK.into(),
            color_mid: BLUE_950.into(),
            color_high: VIOLET_800.into(),
            progress: 0.0,
            speed: 0.4,
            zoom: 4.0,
            background_threshold: 0.0,
            color_low_threshold: 0.0,
            color_mid_threshold: 0.0,
            seed: 69.0,
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
            Node2d::StartMainPassPostProcessing.intern(),
            Self::node_label().intern(),
            Node2d::Bloom.intern(),
        ]
    }
}
