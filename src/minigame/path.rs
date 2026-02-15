use crate::prelude::*;
use bevy::prelude::*;
use bevy::{color::palettes::tailwind::GREEN_800, window::PrimaryWindow};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;
use fever_macros::Lerp;

const DEBUGGER: bool = false;

pub fn path_plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<PathAssets>())
        .add_systems(
            OnEnter(GameState::PhaseOne),
            spawn_phase_one.in_set(MinigameSpawnSystems),
        )
        .add_systems(
            OnEnter(GameState::PhaseTwo),
            spawn_phase_one.in_set(MinigameSpawnSystems),
        )
        .add_systems(Update, (intersect_targets, check_success).chain());
}

#[derive(Clone, Copy, Component, Lerp)]
struct PathOpacity(f32);

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
#[require(LastC)]
struct TargetRoot;

#[derive(Component)]
struct Target(f32);

#[derive(Clone, Copy, Component)]
struct Active;

fn spawn_phase_one(mut commands: Commands, assets: Res<PathAssets>, c: JuliaCoordinates) {
    target(
        &mut commands,
        8.0,
        assets.god.clone(),
        assets.zap.clone(),
        children![
            (Target(10.0), c.transform(0.12925337, 0.7643622)),
            (Target(10.0), c.transform(-0.43770975, 0.9620131)),
            (Target(10.0), c.transform(-0.60180664, -0.07361984)),
            (Target(10.0), c.transform(0.48981094, 0.8597144)),
        ],
    );
    target(
        &mut commands,
        8.0,
        assets.star_ship.clone(),
        assets.power_life.clone(),
        children![
            (Target(10.0), c.transform(0.39360046, 0.22105403)),
            (Target(10.0), c.transform(0.50511175, 0.39056396)),
            (Target(10.0), c.transform(0.48820877, 0.6109084)),
            (Target(10.0), c.transform(0.26229095, 0.81796634)),
            (Target(10.0), c.transform(-0.21299359, 0.64798725)),
        ],
    );
    target(
        &mut commands,
        8.0,
        assets.contrast.clone(),
        assets.zap.clone(),
        children![
            (Target(10.0), c.transform(-0.3643189, -0.6188507)),
            (Target(10.0), c.transform(-0.5354081, -0.6015929)),
            (Target(10.0), c.transform(-0.16873938, -0.64779276)),
            (Target(10.0), c.transform(-0.60802084, -0.17068857)),
            (Target(10.0), c.transform(-0.4474144, 0.020050056)),
        ],
    );
    target(
        &mut commands,
        8.0,
        assets.star_ship.clone(),
        assets.power_life.clone(),
        children![
            (Target(10.0), c.transform(0.074550614, -0.54670715)),
            (Target(10.0), c.transform(-0.21527101, -0.62417215)),
            (Target(10.0), c.transform(0.073577866, -0.39602274)),
            (Target(10.0), c.transform(-0.2700196, -0.27401727)),
        ],
    );

    fn target(
        commands: &mut Commands,
        time: f32,
        texture: Handle<Image>,
        music: Handle<AudioSample>,
        targets: impl Bundle,
    ) {
        let time = if DEBUGGER { 9999999.0 } else { time };
        let tdur = 2.0;
        commands
            .spawn((
                Minigame,
                WinSfx,
                LooseSfx,
                Available,
                MinigameTimer::duration(time),
                ControlsTransition::Mouse,
                ControlTips("FIND THE TARGET"),
                TransitionDuration(tdur / 2.0),
                DespawnOnExit(GameState::PhaseOne),
                TargetRoot,
                targets,
            ))
            .observe(
                move |enter: On<Insert, EnterMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>| {
                    commands.entity(*fractal).insert(ResetFractal).insert((
                        FractalTexture(texture.clone()),
                        Mandelbrot(1),
                        Zoom(1.5),
                    ));

                    commands
                        .entity(enter.entity)
                        .insert((
                            PathOpacity(0.0),
                            SamplerBuilder::new(SamplePlayer::new(music.clone()).looping())
                                .volume(0.0)
                                .lpf(Lpf::MIN)
                                .build(),
                            AnimationTarget::entity(),
                            animations![fade_volume(1.0, 0.8)],
                        ))
                        .with_child((
                            DespawnFinished,
                            AnimationTarget(enter.entity),
                            animations![(
                                Duration(tdur),
                                Keyframe(PathOpacity(1.0)),
                                Easing::SineInOut
                            )],
                        ))
                        .insert_recursive::<Children>(Active);

                    commands.run_system_cached(lock_camera);
                    commands.run_system_cached(force_camera_origin);
                },
            )
            .observe(
                move |exit: On<Insert, ExitMinigame>, mut commands: Commands| {
                    commands
                        .entity(exit.entity)
                        .despawn_related::<Animations>()
                        .remove::<AnimationComponents>()
                        .insert(parallel![
                            (
                                Duration(tdur),
                                Keyframe(PathOpacity(0.0)),
                                Easing::SineInOut
                            ),
                            fade_volume(tdur, 0.0),
                        ]);
                    commands.run_system_cached(unforce_camera_origin);
                },
            );
    }
}

#[derive(Component)]
struct Hovered;

#[derive(Default, Component)]
struct LastC(Vec2);

fn intersect_targets(
    mut commands: Commands,
    mut gizmos: Gizmos,
    targets: Query<(Entity, &Target, &Transform, Has<Hovered>), With<Active>>,
    mut last_c: Single<&mut LastC, (With<TargetRoot>, With<Active>)>,
    mut lpf: Single<&mut Lpf, With<Active>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    assets: Res<PathAssets>,
    coords: JuliaCoordinates,
    path_opacity: Single<&PathOpacity>,
    fade_out: Single<Has<ExitMinigame>, (With<Active>, With<TargetRoot>)>,
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
        let mut c = if *fade_out {
            last_c.0
        } else {
            let c = coords.julia2(w);
            last_c.0 = c;
            c
        };

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

        if path_opacity.0 < 0.5 {
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
    targets: Query<(), (With<Target>, Without<Hovered>, With<Active>)>,
    root: Single<Entity, (With<TargetRoot>, Without<ExitMinigame>, With<Active>)>,
) {
    if targets.is_empty() {
        commands.entity(*root).insert(WonMinigame);
    }
}
