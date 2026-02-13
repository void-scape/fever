use crate::{
    animation::*,
    animations,
    audio::Lpf,
    camera::lock_camera,
    fractal::{Fractal, FractalTexture, JuliaCoordinates, Mandelbrot, Zoom, cmul},
    minigame::{
        Description, Minigame, MinigameRoot, OnVariationEnable, StartTimer, Variation, VariationSet,
    },
    state::GameState,
};
use bevy::{
    color::palettes::tailwind::GREEN_800, ecs::entity_disabling::Disabled, prelude::*,
    window::PrimaryWindow,
};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;
use fever_macros::Lerp;

const DEBUGGER: bool = false;

pub fn plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<PathAssets>())
        .add_systems(OnEnter(GameState::Playing), init_targets)
        .add_systems(OnEnter(Minigame::Path), (lock_camera, fade_opacity))
        .add_systems(
            Update,
            (
                intersect_targets,
                check_success.run_if(in_state(Minigame::Path)),
            )
                .chain(),
        );
}

#[derive(Clone, Copy, Component, Lerp)]
struct PathOpacity(f32);

fn fade_opacity(mut commands: Commands) {
    commands.spawn((
        PathOpacity(0.0),
        AnimationTarget::entity(),
        DespawnOnExit(Minigame::EnterWipe),
        animations![
            (Duration(2.0), Keyframe(PathOpacity(1.0)), Easing::SineInOut),
            blocking_system(|state: Res<State<Minigame>>| { *state.get() != Minigame::Path }),
            (Duration(0.3), Keyframe(PathOpacity(0.0)), Easing::SineInOut),
        ],
    ));
}

#[derive(AssetCollection, Resource)]
struct PathAssets {
    #[asset(path = "images/fractals/god.png")]
    god: Handle<Image>,
    #[asset(path = "images/fractals/star-ship.png")]
    star_ship: Handle<Image>,
    #[asset(path = "images/fractals/contrast.png")]
    contrast: Handle<Image>,
    //
    #[asset(path = "music/power-life.ogg")]
    power_life: Handle<AudioSample>,
    #[asset(path = "music/zap.ogg")]
    zap: Handle<AudioSample>,
    //
    #[asset(path = "sfx/hovered.ogg")]
    hovered: Handle<AudioSample>,
    #[asset(path = "sfx/unhovered.ogg")]
    unhovered: Handle<AudioSample>,
}

#[derive(Component)]
#[require(Transform)]
struct TargetRoot;

#[derive(Component)]
struct Target(f32);

fn init_targets(mut commands: Commands, assets: Res<PathAssets>, c: JuliaCoordinates) {
    let variations = children![
        target(
            &mut commands,
            8.0,
            assets.god.clone(),
            assets.zap.clone(),
            children![
                (Disabled, Target(10.0), c.transform(0.12925337, 0.7643622)),
                (Disabled, Target(10.0), c.transform(-0.43770975, 0.9620131)),
                (
                    Disabled,
                    Target(10.0),
                    c.transform(-0.60180664, -0.07361984)
                ),
                (Disabled, Target(10.0), c.transform(0.48981094, 0.8597144)),
            ],
        ),
        target(
            &mut commands,
            8.0,
            assets.star_ship.clone(),
            assets.power_life.clone(),
            children![
                (Disabled, Target(10.0), c.transform(0.39360046, 0.22105403)),
                (Disabled, Target(10.0), c.transform(0.50511175, 0.39056396)),
                (Disabled, Target(10.0), c.transform(0.48820877, 0.6109084)),
                (Disabled, Target(10.0), c.transform(0.26229095, 0.81796634)),
                (Disabled, Target(10.0), c.transform(-0.21299359, 0.64798725)),
            ],
        ),
        target(
            &mut commands,
            8.0,
            assets.contrast.clone(),
            assets.zap.clone(),
            children![
                (Disabled, Target(10.0), c.transform(-0.3643189, -0.6188507)),
                (Disabled, Target(10.0), c.transform(-0.5354081, -0.6015929)),
                (
                    Disabled,
                    Target(10.0),
                    c.transform(-0.16873938, -0.64779276)
                ),
                (
                    Disabled,
                    Target(10.0),
                    c.transform(-0.60802084, -0.17068857)
                ),
                (Disabled, Target(10.0), c.transform(-0.4474144, 0.020050056)),
            ],
        ),
        target(
            &mut commands,
            8.0,
            assets.star_ship.clone(),
            assets.power_life.clone(),
            children![
                (
                    Disabled,
                    Target(10.0),
                    c.transform(0.074550614, -0.54670715)
                ),
                (
                    Disabled,
                    Target(10.0),
                    c.transform(-0.21527101, -0.62417215)
                ),
                (
                    Disabled,
                    Target(10.0),
                    c.transform(0.073577866, -0.39602274)
                ),
                (Disabled, Target(10.0), c.transform(-0.2700196, -0.27401727)),
            ],
        ),
    ];

    commands.spawn((
        MinigameRoot,
        DespawnOnExit(GameState::Playing),
        Minigame::Path,
        Description::Mouse,
        children![(VariationSet, variations)],
    ));

    fn target(
        commands: &mut Commands,
        time: f32,
        texture: Handle<Image>,
        music: Handle<AudioSample>,
        targets: impl Bundle,
    ) -> impl Bundle {
        let on_start = OnVariationEnable(commands.register_system(
            move |_: In<Entity>,
                  mut commands: Commands,
                  fractal: Single<Entity, With<Fractal>>,
                  mut camera: Single<&mut Transform, With<Camera>>| {
                camera.translation = Vec3::ZERO;
                commands.entity(*fractal).insert((
                    FractalTexture(texture.clone()),
                    Mandelbrot(1),
                    Zoom(1.5),
                ));
            },
        ));

        let time = if DEBUGGER { 9999999.0 } else { time };
        (
            Variation,
            StartTimer(time),
            TargetRoot,
            targets,
            on_start,
            SamplePlayer::new(music)
                .with_volume(Volume::Linear(0.8))
                .looping(),
            sample_effects![LowPassNode {
                frequency: Lpf::distance(1.0, 1.0).0,
            }],
            Lpf::distance(1.0, 1.0),
        )
    }
}

#[derive(Component)]
struct Hovered;

fn intersect_targets(
    mut commands: Commands,
    mut gizmos: Gizmos,
    targets: Query<(Entity, &Target, &Transform, Has<Hovered>)>,
    mut lpf: Single<&mut Lpf, With<TargetRoot>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    assets: Res<PathAssets>,
    coords: JuliaCoordinates,
    path_opacity: Single<&PathOpacity>,
    //
    input: Res<ButtonInput<KeyCode>>,
    mut position: Local<Vec2>,
) {
    for (_, target, transform, is_hovered) in targets.iter() {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.xy()),
            target.0,
            if is_hovered {
                GREEN_800.into()
            } else {
                Color::WHITE
            }
            .with_alpha(path_opacity.0),
        );
    }

    let (camera, camera_transform) = camera.into_inner();
    if let Some(w) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        let mut c = coords.julia2(w);

        if DEBUGGER {
            if input.pressed(KeyCode::ShiftLeft) {
                *position = c;
            }
            if input.just_pressed(KeyCode::KeyA) {
                println!("{c}");
            }
            c = *position;
        }

        let mut path = vec![Vec2::ZERO];
        let mut z = Vec2::ZERO;
        for _ in 0..100 {
            if z.length_squared() > 4.0 * 4.0 {
                break;
            }
            z = cmul(z, z) + c;
            path.push(z);
        }

        if path.len() <= 1 {
            return;
        }

        for points in path.windows(2) {
            gizmos.line_2d(
                coords.world2(points[0]),
                coords.world2(points[1]),
                Color::WHITE.with_alpha(path_opacity.0),
            );
        }

        if path_opacity.0 < 1.0 {
            return;
        }

        let mut hov = false;
        let mut unhov = false;
        'outer: for (entity, target, transform, hovered) in targets.iter() {
            for point in path.iter() {
                let dist = coords
                    .world2(*point)
                    .distance_squared(transform.translation.xy());
                if dist < target.0 * target.0 {
                    if hovered {
                        continue 'outer;
                    }

                    hov = true;
                    commands.entity(entity).insert(Hovered);
                    continue 'outer;
                }
            }
            if hovered {
                unhov = true;
                commands.entity(entity).remove::<Hovered>();
            }
        }

        if hov {
            commands
                .spawn(SamplePlayer::new(assets.hovered.clone()).with_volume(Volume::Linear(0.8)));
        }
        if unhov {
            commands.spawn(
                SamplePlayer::new(assets.unhovered.clone()).with_volume(Volume::Linear(0.8)),
            );
        }

        let len = targets.iter().len() as f32;
        **lpf = Lpf::distance(
            len - targets.iter().filter(|(_, _, _, h)| *h).count() as f32,
            len,
        );
    }
}

fn check_success(
    mut commands: Commands,
    targets: Query<(Entity, &Target, &Transform), Without<Hovered>>,
) {
    if targets.is_empty() {
        commands.set_state(Minigame::Success);
    }
}
