use crate::state::GameState;
use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        view::Hdr,
    },
    sprite_render::{Material2d, Material2dPlugin},
};
use fever_macros::Lerp;

pub fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<FractalUniform>::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnExit(GameState::Loading), (spawn, camera))
        .add_systems(
            Update,
            (
                params,
                move_fractal,
                (cplane, iterations, zoom, opacity, texture),
                fractal,
            )
                .chain(),
        );

    #[cfg(feature = "dev")]
    app.add_systems(Update, log_params);
}

#[derive(Component)]
pub struct Stationary;

pub fn lock_camera(mut commands: Commands, camera: Single<Entity, With<Camera>>) {
    commands.entity(*camera).insert(Stationary);
}

pub fn unlock_camera(mut commands: Commands, camera: Single<Entity, With<Camera>>) {
    commands.entity(*camera).remove::<Stationary>();
}

fn camera(mut commands: Commands) {
    commands.spawn((Camera2d, MovementSensitivity::default(), Hdr));
}

#[derive(Debug, Default, Clone, Asset, TypePath, AsBindGroup, Component)]
pub struct FractalUniform {
    #[uniform(0)]
    pub params: Params,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for FractalUniform {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "fractal.wgsl".into()
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}

#[derive(Component, Deref, DerefMut)]
#[require(CPlane, Iterations, Zoom, Opacity)]
pub struct Fractal(pub FractalUniform);

fn fractal(
    mut materials: ResMut<Assets<FractalUniform>>,
    fractal: Single<&Fractal, Changed<Fractal>>,
) {
    for (_, mat) in materials.iter_mut() {
        mat.params = fractal.params;
        mat.texture = fractal.texture.clone();
    }
}

#[derive(Clone, Component, Deref, DerefMut)]
pub struct FractalTexture(pub Handle<Image>);

fn texture(fractal: Single<(&mut Fractal, &FractalTexture), Changed<FractalTexture>>) {
    let (mut fractal, texture) = fractal.into_inner();
    fractal.texture = texture.0.clone();
}

#[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct Iterations(pub f32);

fn iterations(fractal: Single<(&mut Fractal, &Iterations), Changed<Iterations>>) {
    let (mut fractal, it) = fractal.into_inner();
    fractal.params.iterations = it.0;
}

#[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct CPlane(pub Vec2);

fn cplane(fractal: Single<(&mut Fractal, &CPlane), Changed<CPlane>>) {
    let (mut fractal, c) = fractal.into_inner();
    fractal.params.cx = c.x;
    fractal.params.cy = c.y;
}

#[derive(Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct Zoom(pub f32);

impl Default for Zoom {
    fn default() -> Self {
        Self(1.5)
    }
}

fn zoom(fractal: Single<(&mut Fractal, &Zoom), Changed<Zoom>>) {
    let (mut fractal, zoom) = fractal.into_inner();
    fractal.params.zoom = zoom.0;
}

#[derive(Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct Opacity(pub f32);

impl Default for Opacity {
    fn default() -> Self {
        Self(1.0)
    }
}

fn opacity(fractal: Single<(&mut Fractal, &Opacity), Changed<Opacity>>) {
    let (mut fractal, opacity) = fractal.into_inner();
    fractal.params.opacity = opacity.0;
}

#[derive(Component)]
struct FractalMesh;

const MESH_SIZE: f32 = 1024.0;

fn spawn(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FractalUniform>>,
) {
    let texture = server.load("images/fractals/star-ship.png");
    commands.spawn((
        FractalTexture(texture.clone()),
        Fractal(FractalUniform {
            texture,
            params: Params {
                iterations: 0.0,
                ..Default::default()
            },
        }),
    ));
    commands.spawn((
        FractalMesh,
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(FractalUniform::default())),
        Transform::from_scale(Vec3::splat(MESH_SIZE)).with_translation(Vec3::new(0.0, 0.0, -100.0)),
    ));
}

fn move_fractal(
    mut fractal: Single<&mut Transform, (With<FractalMesh>, Without<Camera2d>)>,
    camera: Single<&Transform, With<Camera2d>>,
) {
    fractal.translation = camera.translation;
}

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Params {
    pub escape_radius: f32,
    pub iterations: f32,
    pub cx: f32,
    pub cy: f32,
    pub zoom: f32,
    pub exponent: f32,
    pub burning_ship: u32,
    pub mandelbrot: u32,
    pub opacity: f32,
    pub _pad: Vec3,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            escape_radius: 2.0,
            iterations: 20.0,
            cx: 0.0,
            cy: 0.0,
            zoom: 1.5,
            exponent: 2.0,
            burning_ship: 0,
            mandelbrot: 0,
            opacity: 1.0,
            _pad: Vec3::ZERO,
        }
    }
}

#[derive(Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct MovementSensitivity(pub f32);

impl Default for MovementSensitivity {
    fn default() -> Self {
        Self(1.0)
    }
}

fn params(
    input: Res<ButtonInput<KeyCode>>,
    transform: Single<
        (&mut Transform, &MovementSensitivity),
        (With<Camera2d>, Without<Stationary>),
    >,
    fractal: Single<(&mut CPlane, &Zoom), With<Fractal>>,
) {
    let codes = [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD];
    let dir = [
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, -1.0),
        Vec2::new(-1.0, 0.0),
        Vec2::new(1.0, 0.0),
    ];
    let inputs = input
        .get_pressed()
        .filter(|i| codes.contains(i))
        .map(|i| dir[codes.iter().position(|c| *c == *i).unwrap()]);

    let (mut transform, sens) = transform.into_inner();
    let (mut cplane, zoom) = fractal.into_inner();
    for dir in inputs {
        let factor = 300.0;
        **cplane += dir / factor * sens.0;
        transform.translation.x += c_to_w(dir.x / factor * sens.0, zoom.0);
        transform.translation.y += c_to_w(dir.y / factor * sens.0, zoom.0);
    }
}

#[cfg(feature = "dev")]
fn log_params(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut assets: ResMut<Assets<FractalUniform>>,
) {
    if input.just_pressed(KeyCode::KeyP) {
        for (_, fractal) in assets.iter_mut() {
            println!("{:#?}", fractal.params);
        }
        use bevy::render::view::window::screenshot::*;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("screenshot.png"));
    }
}

pub fn c_to_w(c: f32, zoom: f32) -> f32 {
    c / zoom * MESH_SIZE / 2.0
}

pub fn w_to_c(w: f32, zoom: f32) -> f32 {
    w * zoom / MESH_SIZE * 2.0
}

pub fn ctransform(cx: f32, cy: f32, zoom: f32) -> Transform {
    Transform::from_translation(Vec3::new(c_to_w(cx, zoom), c_to_w(cy, zoom), 0.0))
}

pub fn cmul(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x)
}
