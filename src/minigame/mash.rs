use crate::prelude::*;
use bevy::{post_process::effect_stack::ChromaticAberration, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;

pub fn mash_plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<MashAssets>())
        .add_systems(
            OnEnter(GameState::PhaseOne),
            spawn_phase_one.in_set(MinigameSpawnSystems),
        )
        .add_systems(
            OnEnter(GameState::PhaseTwo),
            spawn_phase_two.in_set(MinigameSpawnSystems),
        )
        .add_systems(
            Update,
            (mash.before(AnimationSystems::Interpolate), check_success).chain(),
        );
}

#[derive(AssetCollection, Resource)]
struct MashAssets {
    #[asset(path = "images/fractals/odd-julia.png")]
    odd_julia: Handle<Image>,
    #[asset(path = "images/fractals/last-breath.png")]
    last_breath: Handle<Image>,
    #[asset(path = "images/fractals/star-ship.png")]
    star_ship: Handle<Image>,
    #[asset(path = "images/fractals/contrast.png")]
    contrast: Handle<Image>,
    //
    #[asset(path = "music/power-life.ogg")]
    power_life: Handle<AudioSample>,
    #[asset(path = "music/zap.ogg")]
    zap: Handle<AudioSample>,
    #[asset(path = "music/rabbit.ogg")]
    rabbit: Handle<AudioSample>,
    #[asset(path = "music/melo.ogg")]
    melo: Handle<AudioSample>,
}

#[derive(Clone, Copy, Component)]
struct Active;

#[derive(Component)]
struct Count(usize);

#[derive(Component)]
struct StartCount(usize);

#[derive(Component)]
struct ZoomSmooth(f32);

fn spawn_phase_one(mut commands: Commands, assets: Res<MashAssets>) {
    target(
        &mut commands,
        4.0,
        assets.star_ship.clone(),
        assets.zap.clone(),
        Vec2::new(-0.658416, -0.44976),
        15,
        80,
    );
    target(
        &mut commands,
        4.0,
        assets.contrast.clone(),
        assets.power_life.clone(),
        Vec2::new(-0.747543, -0.077061),
        15,
        80,
    );
    target(
        &mut commands,
        4.0,
        assets.odd_julia.clone(),
        assets.rabbit.clone(),
        Vec2::new(-1.40171, 0.0002396),
        15,
        50,
    );
    target(
        &mut commands,
        4.0,
        assets.last_breath.clone(),
        assets.melo.clone(),
        Vec2::new(0.2776412, 0.0093457),
        15,
        40,
    );

    fn target(
        commands: &mut Commands,
        time: f32,
        texture: Handle<Image>,
        music: Handle<AudioSample>,
        target: Vec2,
        count: usize,
        iterations: usize,
    ) {
        let tdur = 2.0;
        commands
            .spawn((
                Minigame,
                WinSfx,
                LooseSfx,
                Available,
                MinigameTimer::duration(time),
                ControlsTransition::Space,
                ControlTips("MASH"),
                TransitionDuration(tdur / 2.0),
                DespawnOnExit(GameState::PhaseOne),
                Count(count),
                StartCount(count),
            ))
            .observe(
                move |enter: On<Insert, EnterMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>| {
                    commands.entity(*fractal).insert(ResetFractal).insert((
                        FractalTexture(texture.clone()),
                        Iterations(iterations as f32),
                        Mandelbrot(1),
                        Zoom(1.5),
                        CPlane(target),
                    ));

                    commands
                        .entity(enter.entity)
                        .insert((Active, ZoomSmooth(1.5)))
                        .insert((
                            SamplerBuilder::new(SamplePlayer::new(music.clone()).looping())
                                .volume(0.0)
                                .lpf(Lpf::MIN)
                                .build(),
                            AnimationTarget::entity(),
                            animations![fade_volume(1.0, 0.8)],
                        ));

                    commands.run_system_cached(lock_camera);
                },
            )
            .observe(
                move |exit: On<Insert, ExitMinigame>, mut commands: Commands| {
                    commands
                        .entity(exit.entity)
                        .despawn_related::<Animations>()
                        .remove::<AnimationComponents>()
                        .insert(parallel![fade_volume(tdur, 0.0),]);
                },
            );
    }
}

fn spawn_phase_two(mut commands: Commands, assets: Res<MashAssets>) {
    target(
        &mut commands,
        3.0,
        assets.star_ship.clone(),
        assets.zap.clone(),
        Vec2::new(-1.753, -0.0205),
        15,
        20,
    );
    target(
        &mut commands,
        3.0,
        assets.contrast.clone(),
        assets.power_life.clone(),
        Vec2::new(0.7369, -1.288),
        15,
        12,
    );
    target(
        &mut commands,
        3.0,
        assets.odd_julia.clone(),
        assets.rabbit.clone(),
        Vec2::new(-0.7536, -1.128),
        15,
        12,
    );
    target(
        &mut commands,
        3.0,
        assets.last_breath.clone(),
        assets.melo.clone(),
        Vec2::new(-1.497, -0.09115),
        15,
        15,
    );

    fn target(
        commands: &mut Commands,
        time: f32,
        texture: Handle<Image>,
        music: Handle<AudioSample>,
        target: Vec2,
        count: usize,
        iterations: usize,
    ) {
        let tdur = 1.0;
        commands
            .spawn((
                Minigame,
                WinSfx,
                LooseSfx,
                Available,
                MinigameTimer::duration(time),
                ControlsTransition::Space,
                ControlTips("MASH"),
                TransitionDuration(tdur / 2.0),
                DespawnOnExit(GameState::PhaseTwo),
                Count(count),
                StartCount(count),
            ))
            .observe(
                move |enter: On<Insert, EnterMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>,
                      completed: Res<CompletedMinigames>,
                      camera: Single<Entity, With<Camera>>| {
                    commands.entity(*camera).insert((
                        ChromaticAberration {
                            intensity: 0.0,
                            ..Default::default()
                        },
                        AberrationIntensity(0.0),
                    ));
                    commands.entity(*fractal).insert(ResetFractal).insert((
                        FractalTexture(texture.clone()),
                        Iterations(iterations as f32),
                        BurningShip(1),
                        Mandelbrot(1),
                        Zoom(1.5),
                        CPlane(target),
                    ));

                    commands
                        .entity(enter.entity)
                        .insert((Active, ZoomSmooth(1.5)))
                        .insert((
                            SamplerBuilder::new(SamplePlayer::new(music.clone()).looping())
                                .volume(0.0)
                                .lpf(Lpf::MIN)
                                .build(),
                            AnimationTarget::entity(),
                            PlaybackSettings::default()
                                .with_speed(0.8 * completed.pitch_modifier()),
                            animations![
                                fade_volume(1.0, 0.8),
                                (
                                    AnimationTarget(*camera),
                                    Duration(tdur / 2.0),
                                    Keyframe(AberrationIntensity(0.3)),
                                    Easing::SineInOut
                                )
                            ],
                        ));

                    commands.run_system_cached(lock_camera);
                },
            )
            .observe(
                move |exit: On<Insert, ExitMinigame>,
                      mut commands: Commands,
                      camera: Single<Entity, With<Camera>>| {
                    commands
                        .entity(exit.entity)
                        .despawn_related::<Animations>()
                        .remove::<AnimationComponents>()
                        .insert(parallel![fade_volume(tdur, 0.0)])
                        .with_child(animations![(
                            AnimationTarget(*camera),
                            Duration(tdur),
                            Easing::SineInOut,
                            Keyframe(AberrationIntensity(0.0)),
                        )]);
                },
            );
    }
}

fn mash(
    time: Res<Time>,
    mut lpf: Single<&mut Lpf, With<Active>>,
    mut count: Single<&mut Count, With<Active>>,
    mut zoom: Single<&mut Zoom, With<Fractal>>,
    mut zoom_smooth: Single<&mut ZoomSmooth, With<Active>>,
    input: Res<ButtonInput<KeyCode>>,
    start_count: Single<
        (&StartCount, Has<ExitMinigame>),
        (With<Minigame>, With<Active>, With<Count>),
    >,
    mut aberration: Option<Single<&mut AberrationIntensity, With<Camera>>>,
) {
    let (start_count, has_exit) = start_count.into_inner();
    if count.0 != 0 && input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        zoom_smooth.0 *= (-0.4f32).exp();
        count.0 = count.0.saturating_sub(1);
        **lpf = Lpf::distance(count.0 as f32 * 0.2, start_count.0 as f32);
    }
    let t = time.delta_secs() * 10.0;
    zoom.0 = zoom.0.lerp(zoom_smooth.0, 1.0 - (-t).exp());
    if !has_exit && let Some(aberration) = &mut aberration {
        aberration.0 = aberration.0.lerp(
            (count.0 as f32 / start_count.0 as f32) * 0.3,
            1.0 - (-t).exp(),
        );
    }
}

fn check_success(
    mut commands: Commands,
    root: Single<(Entity, &Count), (With<Minigame>, Without<ExitMinigame>, With<Active>)>,
) {
    let (entity, count) = root.into_inner();
    if count.0 == 0 {
        commands.entity(entity).insert(WonMinigame);
    }
}
