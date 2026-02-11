use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        view::Hdr,
    },
    sprite_render::{Material2d, Material2dPlugin},
};
use bevy_seedling::spatial::SpatialListener3D;

pub fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<FractalUniform>::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (spawn, camera))
        .add_systems(Update, (params, move_fractal));

    #[cfg(feature = "debug")]
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
    commands.spawn((Camera2d, Hdr, SpatialListener3D));
}

#[derive(Debug, Clone, Asset, TypePath, AsBindGroup, Component)]
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
}

#[derive(Component)]
struct Fractal;

const MESH_SIZE: f32 = 1024.0;
const ZOOM: f32 = 1.5;

fn spawn(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FractalUniform>>,
) {
    let texture = server.load("images/fractals/pickover.png");
    commands.spawn((
        Fractal,
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(FractalUniform {
            params: Params::default(),
            texture,
        })),
        Transform::from_scale(Vec3::splat(MESH_SIZE)).with_translation(Vec3::new(0.0, 0.0, -100.0)),
    ));
}

fn move_fractal(
    mut fractal: Single<&mut Transform, (With<Fractal>, Without<Camera2d>)>,
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
}

impl Default for Params {
    fn default() -> Self {
        Self {
            escape_radius: 2.0,
            iterations: 20.0,
            cx: 0.0,
            cy: 0.0,
            zoom: ZOOM,
            exponent: 2.0,
            burning_ship: 0,
            mandelbrot: 0,
        }
    }
}

fn params(
    input: Res<ButtonInput<KeyCode>>,
    mut assets: ResMut<Assets<FractalUniform>>,
    mut transform: Single<&mut Transform, (With<Camera2d>, Without<Stationary>)>,
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

    for dir in inputs {
        for (_, fractal) in assets.iter_mut() {
            if input.pressed(KeyCode::ShiftLeft) {
                fractal.params.exponent -= exp_to_w(dir.y / 10.0);
                transform.translation.z += exp_to_w(dir.y / 100.0);
            } else {
                let factor = 300.0;
                fractal.params.cx += dir.x / factor;
                fractal.params.cy += dir.y / factor;
                transform.translation.x += c_to_w(dir.x / factor);
                transform.translation.y += c_to_w(dir.y / factor);
            }
        }
    }
}

#[cfg(feature = "debug")]
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

// #[derive(Component)]
// pub struct CPlane;
//
// fn fade_c_plane(
//     mut sprites: Query<(&mut Sprite, &Transform), With<CPlane>>,
//     camera: Single<&Transform, With<Camera>>,
// ) {
//     for (mut sprite, transform) in sprites.iter_mut() {
//         let dist = camera.translation.distance_squared(transform.translation);
//         sprite
//             .color
//             .set_alpha(1.0 - (dist / 1000.0).clamp(0.0, 1.0));
//     }
// }

pub fn c_to_w(c: f32) -> f32 {
    c / ZOOM * MESH_SIZE / 2.0
}

pub fn w_to_c(w: f32) -> f32 {
    w * ZOOM / MESH_SIZE * 2.0
}

fn exp_to_w(exp: f32) -> f32 {
    exp / 2.0
}

pub fn ctransform(cx: f32, cy: f32) -> Transform {
    Transform::from_translation(Vec3::new(c_to_w(cx), c_to_w(cy), 0.0))
}

// fn c_exp_transform(cx: f32, cy: f32, exp: f32) -> Transform {
//     Transform::from_translation(Vec3::new(c_to_w(cx), c_to_w(cy), exp_to_w(exp)))
// }
