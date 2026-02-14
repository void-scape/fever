use crate::{
    camera::{MovementSensitivity, Stationary},
    state::GameState,
};
use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    sprite_render::{Material2d, Material2dPlugin},
    window::PrimaryWindow,
};
use fever_macros::Lerp;

pub fn fractal_plugin(app: &mut App) {
    #[cfg(feature = "dev")]
    app.add_systems(Update, log_params);

    app.add_plugins(Material2dPlugin::<FractalUniform>::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_observer(reset_fractal)
        .add_systems(OnExit(GameState::Loading), spawn)
        .add_systems(
            Update,
            (
                (
                    c_plane,
                    iterations,
                    zoom,
                    opacity,
                    texture,
                    burning_ship,
                    mandelbrot,
                    escape_radius,
                    exponent,
                ),
                move_c_plane,
                fractal,
                sync_fractal_material_with_camera,
            )
                .chain(),
        );
}

macro_rules! new_type_param {
    ($ty:ident, $inner:ty, $fn:ident) => {
        #[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
        pub struct $ty(pub $inner);
        fn $fn(fractal: Single<(&mut Fractal, &$ty), Changed<$ty>>) {
            let (mut fractal, param) = fractal.into_inner();
            fractal.0.params.$fn = param.0;
        }
    };
    ($ty:ident, $inner:ty, $fn:ident, $default:expr) => {
        #[derive(Clone, Copy, Component, Lerp, Deref, DerefMut)]
        pub struct $ty(pub $inner);
        fn $fn(fractal: Single<(&mut Fractal, &$ty), Changed<$ty>>) {
            let (mut fractal, param) = fractal.into_inner();
            fractal.0.params.$fn = param.0;
        }
        impl Default for $ty {
            fn default() -> Self {
                Self($default)
            }
        }
    };
}

#[derive(Default, Component)]
#[require(
    FractalTexture,
    CPlane,
    Iterations,
    Zoom,
    Opacity,
    BurningShip,
    Mandelbrot,
    EscapeRadius,
    Exponent
)]
pub struct Fractal(FractalUniform);

fn fractal(
    mut materials: ResMut<Assets<FractalUniform>>,
    fractal: Single<&Fractal, Changed<Fractal>>,
) {
    for (_, mat) in materials.iter_mut() {
        mat.params = fractal.0.params;
        mat.texture = fractal.0.texture.clone();
    }
}

#[derive(Component)]
pub struct ResetFractal;

fn reset_fractal(fractal: On<Insert, ResetFractal>, mut commands: Commands) {
    commands.entity(fractal.entity).insert((
        FractalTexture::default(),
        CPlane::default(),
        Iterations::default(),
        Zoom::default(),
        Opacity::default(),
        BurningShip::default(),
        Mandelbrot::default(),
        EscapeRadius::default(),
        Exponent::default(),
    ));
}

#[derive(Default, Clone, Component, Deref, DerefMut)]
pub struct FractalTexture(pub Handle<Image>);

fn texture(fractal: Single<(&mut Fractal, &FractalTexture), Changed<FractalTexture>>) {
    let (mut fractal, texture) = fractal.into_inner();
    fractal.0.texture = texture.0.clone();
}

#[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct CPlane(pub Vec2);

fn c_plane(fractal: Single<(&mut Fractal, &CPlane), Changed<CPlane>>) {
    let (mut fractal, param) = fractal.into_inner();
    fractal.0.params.cx = param.x;
    fractal.0.params.cy = param.y;
}

new_type_param!(BurningShip, u32, burning_ship);
new_type_param!(Mandelbrot, u32, mandelbrot);
new_type_param!(Iterations, f32, iterations, 20.0);
new_type_param!(Zoom, f32, zoom, 1.5);
new_type_param!(Opacity, f32, opacity, 1.0);
new_type_param!(EscapeRadius, f32, escape_radius, 2.0);
new_type_param!(Exponent, f32, exponent, 2.0);

#[derive(Component)]
pub struct FractalMesh;

const MESH_SIZE: f32 = 1024.0;

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FractalUniform>>,
) {
    commands.spawn(Fractal::default());
    commands.spawn((
        FractalMesh,
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(FractalUniform::default())),
        Transform::from_scale(Vec3::splat(MESH_SIZE)).with_translation(Vec3::new(0.0, 0.0, -100.0)),
    ));
}

fn sync_fractal_material_with_camera(
    mut fractal: Single<&mut Transform, (With<FractalMesh>, Without<Camera2d>)>,
    camera: Single<&Transform, With<Camera2d>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    fractal.translation.x = camera.translation.x;
    fractal.translation.y = camera.translation.y;
    fractal.scale = Vec3::splat(window.size().min_element());
}

#[derive(Debug, Default, Clone, Asset, TypePath, AsBindGroup, Component)]
struct FractalUniform {
    #[uniform(0)]
    params: Params,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl Material2d for FractalUniform {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/fractal.wgsl".into()
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}

#[derive(Debug, Default, Clone, Copy, ShaderType)]
struct Params {
    escape_radius: f32,
    iterations: f32,
    cx: f32,
    cy: f32,
    zoom: f32,
    exponent: f32,
    burning_ship: u32,
    mandelbrot: u32,
    opacity: f32,
    _pad: Vec3,
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

#[derive(SystemParam)]
pub struct JuliaCoordinates<'w, 's> {
    mesh: Single<'w, 's, &'static Transform, With<FractalMesh>>,
    zoom: Single<'w, 's, &'static Zoom, With<Fractal>>,
}

impl JuliaCoordinates<'_, '_> {
    pub fn world(&self, c: f32) -> f32 {
        c / self.zoom.0 * self.mesh.scale.x / 2.0
    }

    pub fn world2(&self, c: Vec2) -> Vec2 {
        Vec2::new(self.world(c.x), self.world(c.y))
    }

    pub fn julia(&self, w: f32) -> f32 {
        w * self.zoom.0 / self.mesh.scale.x * 2.0
    }

    pub fn julia2(&self, w: Vec2) -> Vec2 {
        Vec2::new(self.julia(w.x), self.julia(w.y))
    }

    pub fn transform(&self, cx: f32, cy: f32) -> Transform {
        Transform::from_translation(Vec3::new(self.world(cx), self.world(cy), 0.0))
    }
}

pub fn cmul(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x)
}

fn move_c_plane(
    input: Res<ButtonInput<KeyCode>>,
    mut cplane: Single<&mut CPlane, With<Fractal>>,
    sens: Single<&MovementSensitivity, (With<Camera>, Without<Stationary>)>,
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
        let factor = 300.0;
        cplane.0 += dir / factor * sens.0;
    }
}
