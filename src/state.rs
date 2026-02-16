use crate::prelude::*;
use bevy::{
    color::palettes::css::RED, post_process::effect_stack::ChromaticAberration, prelude::*,
    text::TextBounds,
};
use bevy_asset_loader::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;
use std::f32::consts::PI;

pub fn state_plugin(app: &mut App) {
    #[cfg(not(feature = "explore"))]
    let state = GameState::Intro;
    #[cfg(feature = "explore")]
    let state = GameState::Explore;
    app.init_state::<GameState>().init_resource::<CompletedMinigames>()
        .add_loading_state(LoadingState::new(GameState::Loading).continue_to_state(state));
}

#[allow(unused)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Intro,
    PhaseOne,
    PhaseTwo,
    Restart(&'static GameState),
    Outro,
    #[cfg(feature = "dev")]
    Explore,
}

#[derive(Default, Resource)]
pub struct CompletedMinigames(pub usize);

impl CompletedMinigames {
    pub fn pitch_modifier(&self) -> f64 {
        1.0 - (self.0 as f64 / 24.0)
    }
}

#[derive(Component)]
struct BackgroundText;

pub fn exit_phase_one(
    mut commands: Commands,
    mut palette: ResMut<TransitionPalette>,
    camera: Single<Entity, With<Camera>>,
    fractal: Single<Entity, With<Fractal>>,
    assets: Res<MinigameAssets>,
    mut completed: ResMut<CompletedMinigames>,
) {
    completed.0 = 0;
    let fractal = *fractal;
    let camera = *camera;
    *palette = TransitionPalette::Blue;
    commands.spawn((
        DespawnFinished,
        SamplerBuilder::new(SamplePlayer::new(assets.big.clone()))
            .volume(0.0)
            .build(),
        PlaybackSpeed(0.7),
        AnimationTarget::entity(),
        animations![
            parallel![
                animations![
                    (
                        AnimationTarget(camera),
                        Duration(2.0),
                        Keyframe(CameraTransitionProgress(0.5))
                    ),
                    system(
                        move |mut commands: Commands, games: Query<Entity, With<Minigame>>| {
                            commands
                                .entity(camera)
                                .remove::<(ChromaticAberration, AberrationIntensity, ForceOrigin)>()
                                .insert(Stationary);
                            for entity in games.iter() {
                                commands.entity(entity).despawn();
                            }
                            commands.spawn((
                                BackgroundText,
                                DespawnOnExit(GameState::PhaseOne),
                                Text2d::new("TIMELESS"),
                                TextColor(RED.into()),
                                TextFont::from_font_size(80.0),
                                TextLayout::new_with_justify(Justify::Center),
                                TextBounds::new_horizontal(MESH_SIZE / 2.0),
                            ));
                        }
                    )
                ],
                fade_volume(8.0, 0.7),
            ],
            system(move |mut commands: Commands, assets: Res<MinigameAssets>| {
                commands.entity(fractal).insert(ResetFractal).insert((
                    Iterations(8.0),
                    FractalTexture(assets.star_ship.clone()),
                    BurningShip(1),
                    CPlane(Vec2::new(0.4888928, 0.08791673)),
                ));
                commands.entity(camera).insert((
                    Translation2D::default(),
                    AberrationIntensity(0.0),
                    ChromaticAberration {
                        intensity: 0.0,
                        ..Default::default()
                    },
                ));
                commands.run_system_cached(unsync_fractal_mesh);
                commands.run_system_cached(narrator_glyph);
                commands.run_system_cached(lock_camera);
            }),
            parallel![
                Duration(10.0),
                animations![
                    Duration(6.0),
                    (
                        Loop::For(3),
                        animations![
                            system(|mut commands: Commands, assets: Res<MinigameAssets>| {
                                commands.spawn((
                                    SamplePlayer::new(assets.yuy.clone())
                                        .with_volume(Volume::Linear(0.2)),
                                    PlaybackSettings::default().with_speed(0.2),
                                    RandomPitch::new(0.1),
                                    sample_effects![FreeverbNode {
                                        room_size: 1.0,
                                        damping: 0.5,
                                        ..Default::default()
                                    }],
                                ));
                            }),
                            parallel![
                                (
                                    AnimationTarget(fractal),
                                    Duration(4.0),
                                    Delta(Rotation(PI)),
                                    Easing::SineInOut,
                                ),
                                animations![
                                    (
                                        AnimationTarget(camera),
                                        Duration(2.0),
                                        Keyframe(AberrationIntensity(0.5)),
                                        Keyframe(CameraTransitionProgress(0.7)),
                                        Easing::SineInOut,
                                    ),
                                    (
                                        AnimationTarget(camera),
                                        Duration(2.0),
                                        Keyframe(AberrationIntensity(0.0)),
                                        Keyframe(CameraTransitionProgress(0.5)),
                                        Easing::SineInOut,
                                    )
                                ],
                            ],
                            Duration(6.0),
                            system(
                                move |mut commands: Commands,
                                      assets: Res<MinigameAssets>,
                                      mut i: Local<usize>, 
                                      background: Query<Entity, With<BackgroundText>>| {
                                    for entity in background.iter() {
                                        commands.entity(entity).despawn();
                                    }
                                    *i += 1;
                                    if *i == 2 {
                                        commands.spawn((
                                            BackgroundText,
                                            Text2d::new("WEAKENING THE STRANDS"),
                                            DespawnOnExit(GameState::PhaseOne),
                                            TextColor(RED.into()),
                                            TextFont::from_font_size(80.0),
                                            TextLayout::new_with_justify(Justify::Center),
                                            TextBounds::new_horizontal(MESH_SIZE / 2.0),
                                        ));
                                        commands.entity(fractal).insert(ResetFractal).insert((
                                            Iterations(4.0),
                                            FractalTexture(assets.odd_julia.clone()),
                                            Exponent(5.0),
                                            CPlane(Vec2::new(0.667, 0.512)),
                                        ));
                                        *i = 0;
                                    } else if *i == 1 {
                                        commands.spawn((
                                            BackgroundText,
                                            Text2d::new("EVOLVING"),
                                            DespawnOnExit(GameState::PhaseOne),
                                            TextColor(RED.into()),
                                            TextFont::from_font_size(80.0),
                                            TextLayout::new_with_justify(Justify::Center),
                                            TextBounds::new_horizontal(MESH_SIZE / 2.0),
                                        ));
                                        commands.entity(fractal).insert(ResetFractal).insert((
                                            Iterations(8.0),
                                            FractalTexture(assets.pl_julia.clone()),
                                            BurningShip(1),
                                            CPlane(Vec2::new(0.803, -1.122)),
                                        ));
                                    } else {
                                        commands.entity(fractal).insert(ResetFractal).insert((
                                            Iterations(8.0),
                                            FractalTexture(assets.star_ship.clone()),
                                            BurningShip(1),
                                            CPlane(Vec2::new(0.4888928, 0.08791673)),
                                        ));
                                    }
                                }
                            ),
                        ]
                    ),
                ],
                animations![
                    unskippable((
                        FadeIn::default(),
                        pretty!(
                        "Awake.|0.25| Falling<0.5>...|0.5|<1> Still falling,|0.25| still awake?|2|"
                    )
                    )),
                    Duration(5.0),
                    unskippable((
                        FadeIn::default(),
                        pretty!("[You](red) learn and in doing so change.|3.0|")
                    )),
                    Duration(4.0),
                    unskippable((
                        FadeIn::default(),
                        pretty!("If there is an end,|1| I draw nearer<0.5>...|2.2|")
                    )),
                ]
            ],
            unskippable(pretty!("I fear that I may never-")),
            system(move |mut commands: Commands| {
                commands
                    .entity(camera)
                    .remove::<(Translation2D, ChromaticAberration, AberrationIntensity)>();
                commands.run_system_cached(sync_fractal_mesh);
            }),
            //
            system(move |mut commands: Commands| {
                commands.set_state(GameState::PhaseTwo);
                commands
                    .spawn((CutTransition, TransitionDuration(0.1)))
                    .insert(RunTransition(None));
            }),
        ],
    ));
}
