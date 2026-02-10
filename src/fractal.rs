use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    sprite_render::{Material2d, Material2dPlugin},
    time::Stopwatch,
};
use bevy_seedling::spatial::SpatialListener3D;

pub fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<FractalUniform>::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (spawn, camera))
        .add_systems(Update, (params, move_fractal, stationary));

    #[cfg(feature = "debug")]
    app.add_systems(Update, log_params);
}

#[derive(Component)]
pub struct Stationary(pub Stopwatch);

fn camera(mut commands: Commands) {
    // commands.trigger(crate::count_down::StartCountDown);
    commands.spawn((Camera2d, Stationary(Stopwatch::new()), SpatialListener3D));
}

fn stationary(time: Res<Time>, mut stationary: Single<&mut Stationary>) {
    stationary.0.tick(time.delta());
}

#[derive(Clone, Asset, TypePath, AsBindGroup, Component)]
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
    let texture = server.load("images/pickover.png");
    let params = Params {
        escape_radius: 2.0,
        iterations: 20.0,
        cx: 0.0,
        cy: 0.0,
        zoom: ZOOM,
        exponent: 2.0,
        burning_ship: 0,
        _pad: 0,
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
pub struct Params {
    pub escape_radius: f32,
    pub iterations: f32,
    pub cx: f32,
    pub cy: f32,
    pub zoom: f32,
    pub exponent: f32,
    pub burning_ship: u32,
    pub _pad: u32,
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
            _pad: 0,
        }
    }
}

fn params(
    input: Res<ButtonInput<KeyCode>>,
    mut assets: ResMut<Assets<FractalUniform>>,
    camera: Single<(&mut Transform, &mut Stationary), With<Camera2d>>,
) {
    let (mut transform, mut stationary) = camera.into_inner();
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
        stationary.0.reset();
        for (_, fractal) in assets.iter_mut() {
            if input.pressed(KeyCode::ShiftLeft) {
                fractal.params.exponent -= exp_to_w(dir.y / 10.0);
                transform.translation.z += exp_to_w(dir.y / 100.0);
            } else {
                fractal.params.cx += dir.x / 100.0;
                fractal.params.cy += dir.y / 100.0;
                transform.translation.x += c_to_w(dir.x / 100.0);
                transform.translation.y += c_to_w(dir.y / 100.0);
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
    (c / ZOOM) * MESH_SIZE / 2.0
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
