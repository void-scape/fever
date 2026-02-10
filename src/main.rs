#[cfg(debug_assertions)]
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::{
    asset::AssetMetaCheck,
    ecs::system::SystemId,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    sprite_render::{Material2d, Material2dPlugin},
    time::Stopwatch,
};
use bevy_seedling::prelude::{hrtf::DistanceAttenuation, *};

mod audio;

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
    .add_plugins(Material2dPlugin::<FractalUniform>::default())
    .insert_resource(ClearColor(Color::BLACK))
    .add_systems(
        Startup,
        (camera, spawn_fractal, spawn_key, spawn_collected_container),
    )
    .add_systems(
        Update,
        (
            stationary,
            params,
            move_fractal,
            fade_c_plane,
            collect_things,
            reach_target,
        )
            .chain(),
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
struct Params {
    escape_radius: f32,
    iterations: f32,
    cx: f32,
    cy: f32,
    zoom: f32,
    exponent: f32,
    burning_ship: u32,
    _pad: u32,
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

fn exp_to_w(exp: f32) -> f32 {
    exp / 2.0
}

fn c_transform(cx: f32, cy: f32) -> Transform {
    Transform::from_translation(Vec3::new(c_to_w(cx), c_to_w(cy), 0.0))
}

fn c_exp_transform(cx: f32, cy: f32, exp: f32) -> Transform {
    Transform::from_translation(Vec3::new(c_to_w(cx), c_to_w(cy), exp_to_w(exp)))
}

#[derive(Component)]
struct Key;

fn spawn_key(mut commands: Commands, server: Res<AssetServer>) {
    let spawn_door = commands.register_system(spawn_door);
    commands.spawn((
        Key,
        Collectable("images/key.png"),
        OnCollect(spawn_door),
        CPlane,
        Sprite::from_image(server.load("images/key.png")),
        c_transform(-0.029999983, -0.7399992).with_scale(Vec3::splat(0.1)),
        //
        SamplePlayer::new(server.load("voice/divine-comedy-grain.ogg"))
            .with_volume(Volume::Linear(0.8))
            .looping(),
        sample_effects![
            FreeverbNode {
                room_size: 0.8,
                damping: 0.8,
                width: 0.5,
                ..Default::default()
            },
            HrtfNode {
                distance_attenuation: DistanceAttenuation {
                    distance_gain_factor: 0.005,
                    ..Default::default()
                },
                ..Default::default()
            }
        ],
    ));
}

#[derive(Component)]
struct Door;

fn spawn_door(mut commands: Commands, server: Res<AssetServer>) {
    let spawn_target_image = commands.register_system(spawn_target_image);
    commands.spawn((
        Door,
        Collectable("images/door.png"),
        OnCollect(spawn_target_image),
        CPlane,
        Sprite::from_image(server.load("images/door.png")),
        c_transform(0.029999983, 0.7399992).with_scale(Vec3::splat(0.1)),
        //
        SamplePlayer::new(server.load("voice/moby-dick-grain.ogg")).looping(),
        sample_effects![
            FreeverbNode {
                room_size: 0.2,
                damping: 1.0,
                width: 0.5,
                ..Default::default()
            },
            HrtfNode {
                distance_attenuation: DistanceAttenuation {
                    distance_gain_factor: 0.005,
                    ..Default::default()
                },
                ..Default::default()
            }
        ],
    ));
}

#[derive(Component)]
struct TargetPosition;

#[derive(Component)]
struct TargetImage;

fn spawn_target_image(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((TargetPosition, c_transform(-0.16120331, -0.6908013)));
    commands.spawn((
        TargetImage,
        ImageNode {
            image: asset_server.load("images/target.png"),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Percent(35.0),
            ..default()
        },
    ));
}

fn reach_target(
    mut commands: Commands,
    target: Single<(Entity, &Transform), With<TargetPosition>>,
    camera: Single<(&Transform, &Stationary), With<Camera>>,
    image: Single<Entity, With<TargetImage>>,
    mut assets: ResMut<Assets<FractalUniform>>,
    server: Res<AssetServer>,
) {
    let (camera_transform, stationary) = camera.into_inner();
    if stationary.0.elapsed_secs() < 1.0 {
        return;
    }

    let (entity, transform) = target.into_inner();
    let dist = camera_transform
        .translation
        .distance_squared(transform.translation);
    if dist < 500.0 {
        commands.queue(audio::sfx_volume("sfx/powerup.ogg", 0.75));
        commands.entity(entity).despawn();
        commands.entity(*image).despawn();

        let texture = server.load("images/inferno.png");
        for (_, fractal) in assets.iter_mut() {
            fractal.params.cx = 0.0;
            fractal.params.cy = 0.0;
            fractal.params.exponent = 2.0;
            fractal.params.burning_ship = 1;
            fractal.texture = texture.clone();
        }
    }
}

#[derive(Component)]
struct CollectedContainer;

fn spawn_collected_container(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },
        CollectedContainer,
    ));
}

#[derive(Component)]
struct Collectable(&'static str);

#[derive(Component)]
struct Collected;

#[derive(Component)]
struct OnCollect(SystemId);

fn collect_things(
    mut commands: Commands,
    things: Query<(Entity, &Transform, &Collectable, Option<&OnCollect>)>,
    camera: Single<(&Transform, &Stationary), With<Camera>>,
    server: Res<AssetServer>,
    container: Single<Entity, With<CollectedContainer>>,
) {
    let (camera_transform, stationary) = camera.into_inner();
    if stationary.0.elapsed_secs() < 1.0 {
        return;
    }

    for (entity, transform, collectable, on) in things.iter() {
        let dist = camera_transform
            .translation
            .distance_squared(transform.translation);
        if dist < 500.0 {
            commands.queue(audio::sfx_volume("sfx/pickup.ogg", 0.75));
            collect_item(&mut commands, &server, *container, collectable.0);
            commands.entity(entity).despawn();
            if let Some(on) = on {
                commands.run_system(on.0);
            }
        }
    }

    fn collect_item(commands: &mut Commands, server: &AssetServer, container: Entity, path: &str) {
        let image = server.load(path.to_string());
        commands.entity(container).with_children(|parent| {
            parent.spawn((
                Collected,
                ImageNode { image, ..default() },
                Node {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    ..default()
                },
            ));
        });
    }
}
