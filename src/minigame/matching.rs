use crate::{
    minigame::prelude::{ControlsTransition, Transition},
    prelude::*,
};
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
    .add_systems(OnEnter(GameState::Playing), init_targets)
    .add_systems(OnEnter(Minigame::Matching), unlock_camera)
    .add_systems(
        Update,
        reach_target.run_if(in_state(Minigame::Matching).and(in_state(Transition::None))),
    );
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

fn init_targets(
    mut commands: Commands,
    assets: Res<MatchingAssets>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let targets = children![
        target(
            &mut commands,
            assets.t0.clone(),
            assets.bong.clone(),
            assets.bands.clone(),
            &mut rng,
            0.1,
            -0.1466666,
            0.83333266,
        ),
        target(
            &mut commands,
            assets.t1.clone(),
            assets.rabbit.clone(),
            assets.contrast.clone(),
            &mut rng,
            0.1,
            0.35999978,
            -0.06666669,
        ),
        target(
            &mut commands,
            assets.t2.clone(),
            assets.bong.clone(),
            assets.star_ship.clone(),
            &mut rng,
            0.1,
            -0.746666,
            -0.21333319,
        ),
        target(
            &mut commands,
            assets.t3.clone(),
            assets.melo.clone(),
            assets.glitch.clone(),
            &mut rng,
            0.1,
            -0.66333276,
            0.42333305,
        ),
    ];

    commands.spawn((
        MinigameRoot,
        Minigame::Matching,
        DespawnOnExit(GameState::Playing),
        children![(VariationSet, targets)],
    ));

    fn target(
        commands: &mut Commands,
        image: Handle<Image>,
        song: Handle<AudioSample>,
        texture: Handle<Image>,
        rng: &mut impl Rng,
        r: f32,
        cx: f32,
        cy: f32,
    ) -> impl Bundle {
        let dc = Vec2::from_angle(rng.random_range(0.0..TAU)) * r;
        let enable = OnVariationEnable(commands.register_system(
            move |root: In<Entity>,
                  mut commands: Commands,
                  fractal: Single<Entity, With<Fractal>>,
                  coords: JuliaCoordinates| {
                commands.entity(*fractal).insert((
                    FractalTexture(texture.clone()),
                    CPlane(Vec2::new(cx, cy) + dc),
                ));

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
                        DespawnOnExit(Minigame::Matching),
                    ))
                    .id();

                commands.entity(*root).with_child((
                    AnimationTarget(*fractal),
                    animations![
                        block_in_state(Transition::Exit),
                        block_in_state(Transition::None),
                        blocking_system(
                            |target: Single<&Transform, With<Target>>,
                             camera: Single<&Transform, With<Camera>>| {
                                camera.translation.distance(target.translation) < MAX_DIST
                            }
                        ),
                        (
                            Duration(0.4),
                            Keyframe(CPlane(Vec2::new(cx, cy))),
                            Easing::SineInOut
                        )
                    ],
                ));

                commands.entity(*root).with_child((
                    Target,
                    coords.transform(cx, cy),
                    // no lpf on music pool :(
                    // MusicPool,
                    PlaybackSpeed(1.0),
                    SamplePlayer::new(song.clone())
                        .with_volume(Volume::Linear(0.8))
                        .looping(),
                    sample_effects![LowPassNode {
                        frequency: Lpf::distance(1.0, 1.0).0,
                    }],
                    Lpf::distance(1.0, 1.0),
                    //
                    AnimationTarget::entity(),
                    animations![
                        Duration(0.3),
                        (
                            AnimationTarget(image),
                            Duration(1.25),
                            Keyframe(ImageColor(Color::WHITE)),
                            Easing::SineInOut
                        ),
                        block_in_state(Transition::None),
                        system(
                            |mut camera: Single<&mut MovementSensitivity, With<Camera>>| {
                                camera.0 = 0.0;
                            }
                        ),
                        parallel![
                            (
                                AnimationTarget(image),
                                Duration(0.3),
                                Keyframe(ImageColor(Color::WHITE.with_alpha(0.0))),
                                Easing::SineInOut
                            ),
                            (
                                Duration(0.2),
                                Keyframe(PlaybackSpeed(0.0)),
                                Easing::SineInOut
                            ),
                        ],
                    ],
                ));
            },
        ));

        (
            Variation,
            TimerDuration(8.0),
            ControlsTransition::Wasd,
            enable,
        )
    }
}

fn reach_target(
    mut commands: Commands,
    target: Single<(&Transform, &mut Lpf), With<Target>>,
    camera: Single<&Transform, With<Camera>>,
) {
    let (transform, mut lpf) = target.into_inner();
    let dist = camera.translation.distance(transform.translation);
    *lpf = Lpf::distance(dist, 32.0);
    if dist < MAX_DIST {
        // TODO: move to final solution
        commands.run_system_cached(success);
    }
}
