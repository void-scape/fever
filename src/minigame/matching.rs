use crate::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use bevy_seedling::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;

pub fn matching_plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(GameState::Loading).load_collection::<MatchingAssets>(),
    )
    .add_systems(
        OnEnter(GameState::PhaseOne),
        spawn_phase_one.in_set(MinigameSpawnSystems),
    )
    .add_systems(
        OnEnter(GameState::PhaseTwo),
        spawn_phase_one.in_set(MinigameSpawnSystems),
    )
    .add_systems(Update, reach_target);
}

#[derive(AssetCollection, Resource)]
struct MatchingAssets {
    #[asset(path = "images/fractals/bands.png")]
    bands: Handle<Image>,
    #[asset(path = "images/fractals/contrast.png")]
    contrast: Handle<Image>,
    #[asset(path = "images/fractals/star-ship.png")]
    star_ship: Handle<Image>,
    #[asset(path = "images/fractals/glitch.png")]
    glitch: Handle<Image>,
    //
    #[asset(path = "images/matching/0.png")]
    t0: Handle<Image>,
    #[asset(path = "images/matching/1.png")]
    t1: Handle<Image>,
    #[asset(path = "images/matching/2.png")]
    t2: Handle<Image>,
    #[asset(path = "images/matching/3.png")]
    t3: Handle<Image>,
    //
    #[asset(path = "music/bong.ogg")]
    bong: Handle<AudioSample>,
    #[asset(path = "music/rabbit.ogg")]
    rabbit: Handle<AudioSample>,
    #[asset(path = "music/melo.ogg")]
    melo: Handle<AudioSample>,
}

const MAX_DIST: f32 = 8.0;

#[derive(Component)]
struct Target;

#[derive(Component)]
struct Active;

fn spawn_phase_one(
    mut commands: Commands,
    assets: Res<MatchingAssets>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    target(
        &mut commands,
        assets.t0.clone(),
        assets.bong.clone(),
        assets.bands.clone(),
        &mut rng,
        0.1,
        -0.1466666,
        0.83333266,
    );
    target(
        &mut commands,
        assets.t1.clone(),
        assets.rabbit.clone(),
        assets.contrast.clone(),
        &mut rng,
        0.1,
        0.35999978,
        -0.06666669,
    );
    target(
        &mut commands,
        assets.t2.clone(),
        assets.bong.clone(),
        assets.star_ship.clone(),
        &mut rng,
        0.1,
        -0.746666,
        -0.21333319,
    );
    target(
        &mut commands,
        assets.t3.clone(),
        assets.melo.clone(),
        assets.glitch.clone(),
        &mut rng,
        0.1,
        -0.66333276,
        0.42333305,
    );

    fn target(
        commands: &mut Commands,
        image: Handle<Image>,
        song: Handle<AudioSample>,
        texture: Handle<Image>,
        rng: &mut impl Rng,
        r: f32,
        cx: f32,
        cy: f32,
    ) {
        let dc = Vec2::from_angle(rng.random_range(0.0..TAU)) * r;
        let tdur = 2.0;

        let image = commands
            .spawn((
                ImageNode {
                    image: image.clone(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.0),
                    top: Val::Px(20.0),
                    width: Val::Percent(35.0),
                    ..default()
                },
                ImageColor(Color::WHITE.with_alpha(0.0)),
            ))
            .id();

        commands
            .spawn((
                Minigame,
                WinSfx,
                LooseSfx,
                Available,
                MinigameTimer::duration(8.0),
                ControlsTransition::Wasd,
                TransitionDuration(tdur / 2.0),
                DespawnOnExit(GameState::PhaseOne),
            ))
            .observe(
                move |enter: On<Insert, EnterMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>,
                      coords: JuliaCoordinates,
                      camera: Single<Entity, With<Camera>>| {
                    commands
                        .entity(*camera)
                        .insert(MovementSensitivity::default());
                    commands.entity(*fractal).insert(ResetFractal).insert((
                        FractalTexture(texture.clone()),
                        CPlane(Vec2::new(cx, cy) + dc),
                    ));

                    commands.entity(image).insert(ImageOf(enter.entity));
                    commands
                        .entity(enter.entity)
                        .insert((
                            Target,
                            Active,
                            coords.transform(cx, cy),
                            SamplerBuilder::new(SamplePlayer::new(song.clone()).looping())
                                .volume(0.0)
                                .lpf(Lpf::MIN)
                                .build(),
                            AnimationTarget::entity(),
                            animations![fade_volume(1.0, 0.7)],
                        ))
                        .with_child((
                            AnimationTarget(image),
                            DespawnFinished,
                            animations![
                                Duration(0.3),
                                (
                                    AnimationTarget(image),
                                    Duration(1.25),
                                    Keyframe(ImageColor(Color::WHITE)),
                                    Easing::SineInOut
                                ),
                            ],
                        ));

                    commands.run_system_cached(unlock_camera);
                },
            )
            .observe(
                move |exit: On<Insert, ExitMinigame>,
                      mut commands: Commands,
                      success: Query<(), With<WonMinigame>>,
                      fractal: Single<Entity, With<Fractal>>| {
                    commands.entity(exit.entity).with_child(parallel![
                        (
                            AnimationTarget(image),
                            Duration(tdur / 3.0),
                            Keyframe(ImageColor(Color::WHITE.with_alpha(0.0))),
                            Easing::SineInOut,
                        ),
                        (AnimationTarget(exit.entity), fade_volume(tdur / 2.0, 0.0)),
                    ]);

                    if success.contains(exit.entity) {
                        commands.entity(exit.entity).with_child((
                            DespawnFinished,
                            animations![
                                system(
                                    |mut camera: Single<&mut MovementSensitivity, With<Camera>>| {
                                        camera.0 = 0.0;
                                    },
                                ),
                                (
                                    AnimationTarget(*fractal),
                                    Duration(tdur / 3.0),
                                    Keyframe(CPlane(Vec2::new(cx, cy))),
                                    Easing::ExponentialOut
                                )
                            ],
                        ));
                    }
                },
            );
    }
}

fn reach_target(
    mut commands: Commands,
    target: Single<
        (Entity, &Transform, &mut Lpf),
        (With<Target>, With<Active>, Without<ExitMinigame>),
    >,
    camera: Single<&Transform, With<Camera>>,
) {
    let (entity, transform, mut lpf) = target.into_inner();
    let dist = camera.translation.distance(transform.translation);
    *lpf = Lpf::distance(dist, 32.0);
    if dist < MAX_DIST {
        commands.entity(entity).insert(WonMinigame);
    }
}
