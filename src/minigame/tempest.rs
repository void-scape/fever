use crate::{
    audio::Lpf,
    camera::{force_camera_origin, lock_camera, unforce_camera_origin},
    fractal::{BurningShip, CPlane, Fractal, FractalTexture, Mandelbrot, Zoom},
    minigame::{Description, Minigame, MinigameRoot, OnVariationEnable, Variation, VariationSet},
    state::GameState,
};
use bevy::{color::palettes::css::YELLOW, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use bevy_seedling::prelude::*;
use rand::Rng;
use std::f32::consts::{PI, TAU};

pub fn plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<TempestAssets>())
        .add_systems(OnEnter(GameState::Playing), init_targets)
        .add_systems(OnEnter(Minigame::Tempest), lock_camera)
        .add_systems(OnEnter(Minigame::Tempest), force_camera_origin)
        .add_systems(OnExit(Minigame::Tempest), unforce_camera_origin)
        .add_systems(
            Update,
            (
                spawner,
                (player, enemy, zoom),
                (lpf, collision),
                check_success,
            )
                .chain()
                .run_if(in_state(Minigame::Tempest)),
        );
}

#[derive(AssetCollection, Resource)]
struct TempestAssets {
    #[asset(path = "images/fractals/last-breath.png")]
    last_breath: Handle<Image>,
    #[asset(path = "images/fractals/star-ship.png")]
    star_ship: Handle<Image>,
    //
    #[asset(path = "music/wash-over.ogg")]
    wash_over: Handle<AudioSample>,
    #[asset(path = "music/fall-apart.ogg")]
    fall_apart: Handle<AudioSample>,
    #[asset(path = "music/melo.ogg")]
    melo: Handle<AudioSample>,
}

#[derive(Component)]
#[require(Transform)]
struct SceneRoot;

fn init_targets(
    mut commands: Commands,
    assets: Res<TempestAssets>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let p0 = rng.random_range(0.0..TAU);
    let p1 = rng.random_range(0.0..TAU);
    let p2 = rng.random_range(0.0..TAU);
    let scenes = children![
        scene(
            &mut commands,
            assets.last_breath.clone(),
            assets.wash_over.clone(),
            move || children![
                (
                    Player(p0),
                    Sprite::from_color(YELLOW, Vec2::new(50.0, 20.0)),
                    Transform::from_xyz(RADIUS, 0.0, 1.0),
                ),
                Spawner {
                    timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                    wave: 0,
                    enemies: 8,
                }
            ]
        ),
        scene(
            &mut commands,
            assets.star_ship.clone(),
            assets.fall_apart.clone(),
            move || children![
                (
                    Player(p1),
                    Sprite::from_color(YELLOW, Vec2::new(50.0, 20.0)),
                    Transform::from_xyz(RADIUS, 0.0, 1.0),
                ),
                Spawner {
                    timer: Timer::from_seconds(0.4, TimerMode::Repeating),
                    wave: 0,
                    enemies: 10,
                }
            ]
        ),
        scene(
            &mut commands,
            assets.star_ship.clone(),
            assets.melo.clone(),
            move || children![
                (
                    Player(p2),
                    Sprite::from_color(YELLOW, Vec2::new(50.0, 20.0)),
                    Transform::from_xyz(RADIUS, 0.0, 1.0),
                ),
                Spawner {
                    timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                    wave: 0,
                    enemies: 7,
                }
            ]
        )
    ];

    commands.spawn((
        MinigameRoot,
        DespawnOnExit(GameState::Playing),
        Minigame::Tempest,
        Description::Wasd,
        children![(VariationSet, scenes)],
    ));

    fn scene<T>(
        commands: &mut Commands,
        texture: Handle<Image>,
        music: Handle<AudioSample>,
        mut colliders: impl FnMut() -> T + Send + Sync + 'static,
    ) -> impl Bundle
    where
        T: Bundle,
    {
        let on_enable = OnVariationEnable(commands.register_system(
            move |_: In<Entity>, mut commands: Commands, fractal: Single<Entity, With<Fractal>>| {
                commands.entity(*fractal).insert((
                    FractalTexture(texture.clone()),
                    CPlane(Vec2::new(-1.741702, -0.052180)),
                    BurningShip(1),
                    Mandelbrot(1),
                    Zoom(0.05),
                ));

                commands.spawn((
                    DespawnOnExit(Minigame::EnterWipe),
                    Transform::default(),
                    Visibility::default(),
                    colliders(),
                ));
            },
        ));
        (
            Variation,
            SceneRoot,
            on_enable,
            SamplePlayer::new(music)
                .with_volume(Volume::Linear(0.8))
                .looping(),
            sample_effects![LowPassNode {
                frequency: 20_000.0
            }],
            Lpf(20_000.0),
        )
    }
}

#[derive(Component)]
struct Player(f32);

const RADIUS: f32 = 250.0;
const ROTATION_SPEED: f32 = 1.5;
const ENEMY_ACCEL: f32 = 5.0;

fn player(
    player: Single<(&mut Transform, &mut Player)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (mut transform, mut player) = player.into_inner();
    let dir = (keys.pressed(KeyCode::KeyA) as i32 - keys.pressed(KeyCode::KeyD) as i32) as f32;
    player.0 -= dir * ROTATION_SPEED * time.delta_secs();
    transform.translation.x = player.0.cos() * RADIUS;
    transform.translation.y = player.0.sin() * RADIUS;
    transform.rotation = Quat::from_rotation_z(player.0 + PI / 2.0);
}

#[derive(Component)]
struct Spawner {
    timer: Timer,
    wave: usize,
    enemies: usize,
}

fn spawner(mut commands: Commands, mut query: Query<(Entity, &mut Spawner)>, time: Res<Time>) {
    for (entity, mut spawner) in &mut query {
        if spawner.wave >= 3 {
            commands.entity(entity).despawn();
            continue;
        }

        if spawner.timer.tick(time.delta()).just_finished() {
            let offset = spawner.wave as f32 * PI * 0.4;
            for i in 0..spawner.enemies {
                let angle = i as f32 / spawner.enemies as f32 * 2.0 * PI + offset;
                commands.spawn((
                    Enemy { angle, vel: 0.0 },
                    Sprite::from_color(Color::WHITE, Vec2::new(50.0, 50.0)),
                    Transform::from_xyz(0.0, 0.0, 1.0),
                    DespawnOnExit(Minigame::EnterWipe),
                ));
            }
            spawner.wave += 1;
        }
    }
}

#[derive(Component)]
struct Enemy {
    angle: f32,
    vel: f32,
}

fn enemy(
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut enemy) in enemies.iter_mut() {
        enemy.vel += ENEMY_ACCEL * time.delta_secs();
        let dist = transform.translation.xy().length();
        transform.translation.x = enemy.angle.cos() * (dist + enemy.vel);
        transform.translation.y = enemy.angle.sin() * (dist + enemy.vel);
        transform.scale = Vec3::splat((dist / RADIUS).clamp(0.1, 1.5));
        if dist > RADIUS * 1.5 {
            commands.entity(entity).despawn();
        }
    }
}

fn zoom(mut zoom: Single<&mut Zoom, With<Fractal>>, time: Res<Time>) {
    zoom.0 *= (-time.delta_secs()).exp();
}

fn lpf(mut lpf: Single<&mut Lpf, With<SceneRoot>>, enemies: Query<&Transform, With<Enemy>>) {
    let closest = enemies
        .iter()
        .map(|t| (t.translation.xy().length_squared() - RADIUS * RADIUS).abs())
        .reduce(f32::min);
    if let Some(closest) = closest {
        **lpf = Lpf::distance(closest.sqrt() * 1.5, RADIUS);
    }
}

fn collision(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    for transform in enemies.iter() {
        if transform
            .translation
            .xy()
            .distance_squared(player.translation.xy())
            < 15.0 * 15.0
        {
            commands.set_state(Minigame::Failure);
            return;
        }
    }
}

fn check_success(
    mut commands: Commands,
    enemies: Query<(), With<Enemy>>,
    spawner: Option<Single<(), With<Spawner>>>,
) {
    if spawner.is_none() && enemies.is_empty() {
        commands.set_state(Minigame::Success);
    }
}
