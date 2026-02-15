use crate::prelude::*;
use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;

const SENSITIVITY: f32 = 1000.0;

pub fn sweep_plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<SweepAssets>())
        .add_systems(
            OnEnter(GameState::PhaseOne),
            spawn_phase_one.in_set(MinigameSpawnSystems),
        )
        .add_systems(
            OnEnter(GameState::PhaseTwo),
            spawn_phase_one.in_set(MinigameSpawnSystems),
        )
        .add_systems(Update, (mash, check_success).chain());
}

#[derive(AssetCollection, Resource)]
struct SweepAssets {
    #[asset(path = "images/fractals/bands.png")]
    bands: Handle<Image>,
    #[asset(path = "images/fractals/inferno.png")]
    inferno: Handle<Image>,
    #[asset(path = "images/fractals/glitch.png")]
    glitch: Handle<Image>,
    #[asset(path = "images/fractals/blind-magma.png")]
    blind_magma: Handle<Image>,
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

#[derive(Component, Default)]
struct MouseAccum(f32);

fn spawn_phase_one(mut commands: Commands, assets: Res<SweepAssets>) {
    target(
        &mut commands,
        4.0,
        assets.bands.clone(),
        assets.zap.clone(),
        Vec2::new(0.44380, 0.373566),
        15,
        80,
    );
    target(
        &mut commands,
        4.0,
        assets.inferno.clone(),
        assets.power_life.clone(),
        Vec2::new(0.233425, 0.5347365),
        15,
        80,
    );
    target(
        &mut commands,
        4.0,
        assets.glitch.clone(),
        assets.rabbit.clone(),
        Vec2::new(-0.171291, -0.649680),
        15,
        50,
    );
    target(
        &mut commands,
        4.0,
        assets.blind_magma.clone(),
        assets.melo.clone(),
        Vec2::new(-1.76561, -0.009970),
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
                ControlsTransition::Mouse,
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
                        .insert((Active, ZoomSmooth(1.5), MouseAccum::default()))
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

fn mash(
    time: Res<Time>,
    mut input: MessageReader<MouseMotion>,
    mut lpf: Single<&mut Lpf, With<Active>>,
    mut count: Single<&mut Count, With<Active>>,
    mut zoom: Single<&mut Zoom, With<Fractal>>,
    mut zoom_smooth: Single<&mut ZoomSmooth, With<Active>>,
    mut accum: Single<&mut MouseAccum, With<Active>>,
    start_count: Single<&StartCount, (With<Minigame>, With<Active>, With<Count>)>,
) {
    let mut delta = 0.0;
    for ev in input.read() {
        delta += ev.delta.length();
    }
    accum.0 += delta;

    if count.0 != 0 && accum.0 >= SENSITIVITY {
        zoom_smooth.0 *= (-0.4f32).exp();
        count.0 = count.0.saturating_sub(1);
        **lpf = Lpf::distance(count.0 as f32 * 0.2, start_count.0 as f32);
        accum.0 = 0.0;
    }

    let t = time.delta_secs() * 10.0;
    zoom.0 = zoom.0.lerp(zoom_smooth.0, 1.0 - (-t).exp());
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
