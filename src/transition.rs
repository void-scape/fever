use bevy::{
    core_pipeline::{
        core_2d::graph::Node2d,
        fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
    },
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_graph::{InternedRenderLabel, RenderLabel},
        render_resource::ShaderType,
    },
};
use fever_macros::Lerp;

pub fn plugin(app: &mut App) {
    app.add_plugins(FullscreenMaterialPlugin::<Transition>::default())
        .add_systems(Update, transition_progress);
}

#[derive(Default, Clone, Copy, Lerp, Component, Deref, DerefMut)]
pub struct TransitionProgress(pub f32);

fn transition_progress(
    time: Res<Time>,
    args: Single<(&mut Transition, &TransitionProgress), Changed<TransitionProgress>>,
) {
    let (mut transition, progress) = args.into_inner();
    transition.time += time.delta_secs();
    transition.progress = progress.0;
    transition.background_threshold = (1.0 - progress.0 * 2.0).abs() - 0.5;
    transition.color_threshold = (-4.0 + progress.0 * 8.0).abs().min(1.0) * 0.48;
}

#[derive(Clone, Copy, Component, ExtractComponent, ShaderType)]
#[require(TransitionProgress)]
pub struct Transition {
    pixelation: Vec2,
    color: LinearRgba,
    progress: f32,
    speed: f32,
    zoom: f32,
    background_threshold: f32,
    color_threshold: f32,
    seed: f32,
    time: f32,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            pixelation: Vec2::splat(1.0),
            color: Color::BLACK.into(),
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

impl FullscreenMaterial for Transition {
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
