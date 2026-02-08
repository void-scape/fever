use crate::markov::Markov;
#[cfg(debug_assertions)]
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::{
    asset::AssetMetaCheck,
    input::mouse::MouseWheel,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    sprite_render::{Material2d, Material2dPlugin},
    time::Stopwatch,
};
use bevy_seedling::prelude::{hrtf::DistanceAttenuation, *};

mod markov;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics on web build on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Window {
                    title: "fever".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
        bevy_seedling::SeedlingPlugin::default(),
    ))
    .add_plugins((
        Material2dPlugin::<FractalUniform>::default(),
        markov::plugin,
    ))
    .insert_resource(ClearColor(Color::BLACK))
    .add_systems(Startup, (camera, spawn_fractal, spawn_key))
    .add_systems(
        Update,
        (stationary, params, move_fractal, fade_c_plane, collect_key).chain(),
    );

    #[cfg(debug_assertions)]
    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            ..Default::default()
        },
    })
    .add_systems(Update, (close_on_escape, log_params));

    app.run();
}

#[cfg(debug_assertions)]
fn close_on_escape(mut writer: MessageWriter<AppExit>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        writer.write(AppExit::Success);
    }
}

#[derive(Component)]
struct Stationary(Stopwatch);

fn camera(mut commands: Commands) {
    commands.spawn((Camera2d, Stationary(Stopwatch::new()), SpatialListener3D));
}

fn stationary(time: Res<Time>, mut stationary: Single<&mut Stationary>) {
    stationary.0.tick(time.delta());
}

#[derive(Clone, Asset, TypePath, AsBindGroup)]
struct FractalUniform {
    #[uniform(0)]
    params: Params,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl Material2d for FractalUniform {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "fractal.wgsl".into()
    }
}

#[derive(Component)]
struct Fractal;

const MESH_SIZE: f32 = 1024.0;
const ZOOM: f32 = 1.5;

fn spawn_fractal(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FractalUniform>>,
) {
    let texture = server.load("vg.jpg");
    let params = Params {
        escape_radius: 2.0,
        iterations: 20.0,
        cx: 0.0,
        cy: 0.0,
        zoom: ZOOM,
        _pad: Default::default(),
    };

    commands.spawn((
        Fractal,
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(FractalUniform { params, texture })),
        Transform::from_scale(Vec3::splat(MESH_SIZE)),
    ));
}

fn move_fractal(
    mut fractal: Single<&mut Transform, (With<Fractal>, Without<Camera2d>)>,
    camera: Single<&Transform, With<Camera2d>>,
) {
    fractal.translation = camera.translation;
}

#[derive(Debug, Clone, Copy, ShaderType)]
struct Params {
    escape_radius: f32,
    iterations: f32,
    cx: f32,
    cy: f32,
    zoom: f32,
    _pad: UVec3,
}

fn params(
    mut input: MessageReader<MouseWheel>,
    mut assets: ResMut<Assets<FractalUniform>>,
    camera: Single<(&mut Transform, &mut Stationary), With<Camera2d>>,
) {
    let (mut transform, mut stationary) = camera.into_inner();
    for mw in input.read() {
        stationary.0.reset();
        for (_, fractal) in assets.iter_mut() {
            fractal.params.cx += mw.x / 1000.0;
            fractal.params.cy += mw.y / 1000.0;
            transform.translation.x += c_to_w(mw.x / 1000.0);
            transform.translation.y += c_to_w(mw.y / 1000.0);
        }
    }
}

fn log_params(input: Res<ButtonInput<KeyCode>>, mut assets: ResMut<Assets<FractalUniform>>) {
    if input.just_pressed(KeyCode::KeyP) {
        for (_, fractal) in assets.iter_mut() {
            println!("{:#?}", fractal.params);
        }
    }
}

#[derive(Component)]
struct CPlane;

fn fade_c_plane(
    mut sprites: Query<(&mut Sprite, &Transform), With<CPlane>>,
    camera: Single<&Transform, With<Camera>>,
) {
    for (mut sprite, transform) in sprites.iter_mut() {
        let dist = camera.translation.distance_squared(transform.translation);
        sprite
            .color
            .set_alpha(1.0 - (dist / 1000.0).clamp(0.0, 1.0));
    }
}

fn c_to_w(c: f32) -> f32 {
    (c / ZOOM) * MESH_SIZE / 2.0
}

fn c_transform(cx: f32, cy: f32) -> Transform {
    Transform::from_translation(Vec3::new(c_to_w(cx), c_to_w(cy), 0.0))
}

#[derive(Component)]
struct Key;

fn spawn_key(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        Key,
        CPlane,
        Sprite::from_image(server.load("key.png")),
        c_transform(-0.029999983, -0.7399992).with_scale(Vec3::splat(0.1)),
        //
        SamplePlayer::new(server.load("dc.ogg")).looping(),
        sample_effects![HrtfNode {
            distance_attenuation: DistanceAttenuation {
                distance_gain_factor: 0.005,
                ..Default::default()
            },
            ..Default::default()
        }],
    ));
}

fn collect_key(
    mut commands: Commands,
    key: Single<(Entity, &Transform), With<Key>>,
    camera: Single<(&Transform, &Stationary), With<Camera>>,
    markov: Single<&Markov>,
) {
    let (camera_transform, stationary) = camera.into_inner();
    if stationary.0.elapsed_secs() < 1.0 {
        return;
    }

    let (entity, transform) = key.into_inner();
    let dist = camera_transform
        .translation
        .distance_squared(transform.translation);
    if dist < 500.0 {
        commands.entity(entity).despawn();
        let mut str = String::new();
        markov.write_text(&mut str, 100);
        println!("{str}");
    }
}
