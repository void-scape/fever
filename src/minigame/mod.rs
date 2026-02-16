use crate::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;
use rand::{Rng, seq::IteratorRandom};
use std::collections::VecDeque;

mod dream;
mod mash;
mod matching;
mod path;
mod sweep;
pub mod typing;

pub fn minigame_plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(GameState::Loading).load_collection::<MinigameAssets>(),
    )
    .add_plugins((
        sweep::sweep_plugin,
        mash::mash_plugin,
        path::path_plugin,
        dream::dream_plugin,
        typing::typing_plugin,
        matching::matching_plugin,
    ))
    .init_resource::<MinigameQueue>()
    .add_observer(insert_timer_text)
    .add_observer(minigame_sequence)
    .add_observer(won_minigame)
    .add_observer(lost_minigame)
    .add_observer(run_transition)
    .add_observer(exhausted_minigames)
    .add_systems(Update, minigame_timer)
    .add_systems(
        OnEnter(GameState::PhaseOne),
        start_minigames.after(MinigameSpawnSystems),
    )
    .add_systems(
        OnEnter(GameState::PhaseTwo),
        start_minigames.after(MinigameSpawnSystems),
    );
}

fn start_minigames(mut commands: Commands) {
    commands.spawn(Minigame).insert(WonMinigame).despawn();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct MinigameSpawnSystems;

#[derive(AssetCollection, Resource)]
pub struct MinigameAssets {
    #[asset(path = "third-party/wasd.png")]
    pub wasd: Handle<Image>,
    #[asset(path = "third-party/mouse.png")]
    pub mouse: Handle<Image>,
    #[asset(path = "third-party/keyboard.png")]
    keyboard: Handle<Image>,
    #[asset(path = "third-party/space.png")]
    space: Handle<Image>,
    //
    #[asset(path = "images/fractals/noise1-last-breath.png")]
    pub noise1: Handle<Image>,
    #[asset(path = "images/fractals/noise2-last-breath.png")]
    pub noise2: Handle<Image>,
    #[asset(path = "images/fractals/noise3-last-breath.png")]
    pub noise3: Handle<Image>,
    #[asset(path = "images/fractals/noise4-last-breath.png")]
    pub noise4: Handle<Image>,
    #[asset(path = "images/fractals/last-breath.png")]
    pub last_breath: Handle<Image>,
    #[asset(path = "images/fractals/pl-julia.png")]
    pub pl_julia: Handle<Image>,
    #[asset(path = "images/fractals/odd-julia.png")]
    pub odd_julia: Handle<Image>,
    #[asset(path = "images/fractals/star-ship.png")]
    pub star_ship: Handle<Image>,
    //
    #[asset(path = "sfx/cut.ogg")]
    pub cut: Handle<AudioSample>,
    #[asset(path = "sfx/control.ogg")]
    pub control: Handle<AudioSample>,
    #[asset(path = "sfx/timeout.ogg")]
    timeout: Handle<AudioSample>,
    #[asset(path = "sfx/succeed.ogg")]
    succeed: Handle<AudioSample>,
    #[asset(path = "third-party/big.ogg")]
    pub big: Handle<AudioSample>,
    //
    #[asset(path = "sfx/narrator-glyph.ogg")]
    pub narrator_glyph: Handle<AudioSample>,
    #[asset(path = "sfx/presence-glyph.ogg")]
    pub presence_glyph: Handle<AudioSample>,
    #[asset(path = "sfx/yuy.ogg")]
    pub yuy: Handle<AudioSample>,
    //
    #[asset(path = "third-party/font.otf")]
    pub font: Handle<Font>,
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Minigame;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct MinigameSequence;

fn minigame_sequence(
    insert: On<Insert, ExitMinigame>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
    root: Query<&Children>,
    minigames: Query<Entity, (With<Minigame>, Without<ExitMinigame>)>,
) {
    if let Ok(parent) = parents.get(insert.entity)
        && let Ok(children) = root.get(parent.0)
        && let Some(entity) = minigames.iter_many(children).next()
    {
        commands.entity(entity).insert(Available);
    }
}

#[derive(Component)]
pub struct Available;

#[derive(Component)]
pub struct AvailableAfter(pub usize);

fn available_after(mut commands: Commands, mut entities: Query<(Entity, &mut AvailableAfter)>) {
    for (entity, mut after) in entities.iter_mut() {
        after.0 = after.0.saturating_sub(1);
        if after.0 <= 1 {
            commands
                .entity(entity)
                .remove::<AvailableAfter>()
                .insert(Available);
        }
    }
}

#[derive(Default, Resource, Component, Deref, DerefMut)]
pub struct MinigameQueue(pub VecDeque<Entity>);

#[derive(Component)]
pub struct WonMinigame;

#[derive(Component)]
pub struct WinSfx;

fn won_minigame(
    inserted: On<Insert, WonMinigame>,
    mut commands: Commands,
    sfx: Query<(), With<WinSfx>>,
    valid_minigame: Query<(), With<Minigame>>,
    available_minigames: Query<Entity, (With<Minigame>, With<Available>)>,
    mut queue: ResMut<MinigameQueue>,
    assets: Res<MinigameAssets>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut completed: ResMut<CompletedMinigames>,
) {
    completed.0 += 1;
    if valid_minigame.get(inserted.entity).is_err() {
        panic!("inserted `WonMinigame` into a non `Minigame` entity");
    }
    commands
        .entity(inserted.entity)
        .remove::<Available>()
        .insert(ExitMinigame);
    if sfx.contains(inserted.entity) {
        commands.spawn((
            SamplePlayer::new(assets.succeed.clone()).with_volume(Volume::Linear(0.4)),
            PlaybackSettings::default().with_speed(0.8),
            WinSfx,
        ));
    }
    commands.run_system_cached(available_after);
    if let Some(next) = queue.pop_front() {
        commands
            .entity(next)
            .insert(RunTransition(Some(inserted.entity)));
        return;
    }
    if let Some(next) = available_minigames
        .iter()
        .filter(|e| *e != inserted.entity)
        .choose(&mut rng)
    {
        commands
            .entity(next)
            .insert(RunTransition(Some(inserted.entity)));
    } else {
        commands.trigger(ExhaustedMinigames);
    }
}

#[derive(Component)]
pub struct LostMinigame;

#[derive(Component)]
pub struct LooseSfx;

fn lost_minigame(
    inserted: On<Insert, LostMinigame>,
    mut commands: Commands,
    sfx: Query<(), With<LooseSfx>>,
    valid_minigame: Query<(), With<Minigame>>,
    available_minigames: Query<Entity, (With<Minigame>, With<Available>)>,
    mut queue: ResMut<MinigameQueue>,
    assets: Res<MinigameAssets>,
    mut count: Local<usize>,
    state: Res<State<GameState>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut completed: ResMut<CompletedMinigames>,
) {
    completed.0 += 1;
    if valid_minigame.get(inserted.entity).is_err() {
        panic!("inserted `LostMinigame` into a non `Minigame` entity");
    }
    commands
        .entity(inserted.entity)
        .remove::<Available>()
        .insert(ExitMinigame);
    if sfx.contains(inserted.entity) {
        commands.spawn((
            SamplePlayer::new(assets.timeout.clone()).with_volume(Volume::Linear(0.4)),
            PlaybackSettings::default().with_speed(0.8),
            LooseSfx,
        ));
    }
    *count += 1;
    if *count >= 3 {
        *count = 0;
        queue.clear();
        commands.set_state(GameState::Restart(match state.get() {
            GameState::PhaseOne => &GameState::PhaseOne,
            GameState::PhaseTwo => &GameState::PhaseTwo,
            state => panic!("invalid game state for reset {state:?}"),
        }));
        return;
    }
    commands.run_system_cached(available_after);
    if let Some(next) = queue.pop_front() {
        commands
            .entity(next)
            .insert(RunTransition(Some(inserted.entity)));
        return;
    }
    if let Some(next) = available_minigames
        .iter()
        .filter(|e| *e != inserted.entity)
        .choose(&mut rng)
    {
        commands
            .entity(next)
            .insert(RunTransition(Some(inserted.entity)));
    } else {
        commands.trigger(ExhaustedMinigames);
    }
}

#[derive(Component)]
pub struct MinigameTimer(Timer);

impl MinigameTimer {
    pub fn duration(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Once))
    }
}

fn insert_timer_text(
    inserted: On<Insert, EnterMinigame>,
    mut commands: Commands,
    timer: Query<(), With<MinigameTimer>>,
) {
    if timer.contains(inserted.entity) {
        commands.entity(inserted.entity).insert((
            Text::default(),
            TextFont::from_font_size(40.0),
            TextColor::default(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            },
        ));
    }
}

fn minigame_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<
        (
            Entity,
            &mut MinigameTimer,
            &mut Text,
            &mut TextColor,
            Has<ExitMinigame>,
        ),
        With<EnterMinigame>,
    >,
) {
    for (entity, mut timer, mut text, mut color, done) in timers.iter_mut() {
        if done {
            commands
                .entity(entity)
                .remove::<(MinigameTimer, Text, TextColor)>();
        }
        timer.0.tick(time.delta());
        if timer.0.is_finished() {
            commands
                .entity(entity)
                .remove::<(MinigameTimer, Text, TextColor)>()
                .insert(LostMinigame);
        }

        text.0 = format!("{:.1}", timer.0.remaining_secs());
        if timer.0.remaining_secs() <= 2.0 {
            color.0 = RED.into()
        };
    }
}

#[derive(Debug, Component)]
#[require(TransitionDuration)]
pub enum ControlsTransition {
    Keyboard,
    Mouse,
    Wasd,
    Space,
}

#[derive(Component)]
pub struct ControlTips(pub &'static str);

#[derive(Debug, Component)]
#[require(TransitionDuration)]
pub struct CutTransition;

#[derive(Component)]
pub struct NoCutSfx;

#[derive(Component)]
pub struct TransitionDuration(pub f32);

impl Default for TransitionDuration {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, Component)]
pub struct RunTransition(pub Option<Entity>);

fn run_transition(
    inserted: On<Insert, RunTransition>,
    mut commands: Commands,
    run: Query<&RunTransition>,
    transitions: Query<(
        Option<&ControlsTransition>,
        Option<&CutTransition>,
        Has<NoCutSfx>,
        Option<&ControlTips>,
        &TransitionDuration,
    )>,
    camera: Single<Entity, With<CameraTransition>>,
    mut palette: ResMut<TransitionPalette>,
    minigame_state: Query<(Has<WonMinigame>, Has<LostMinigame>)>,
    assets: Res<MinigameAssets>,
) {
    let runner = run.get(inserted.entity).unwrap();
    let (controls, cut, no_cut_sfx, tips, duration) = transitions.get(inserted.entity).unwrap();
    if let Some(controls) = controls {
        let prev_entity = runner.0;
        if let Some(prev) = prev_entity {
            let (won, lost) = minigame_state.get(prev).unwrap();
            if won {
                *palette = TransitionPalette::Blue;
            } else if lost {
                *palette = TransitionPalette::Red;
            }
        } else {
            *palette = TransitionPalette::Blue;
        }

        let image = match *controls {
            ControlsTransition::Wasd => assets.wasd.clone(),
            ControlsTransition::Mouse => assets.mouse.clone(),
            ControlsTransition::Keyboard => assets.keyboard.clone(),
            ControlsTransition::Space => assets.space.clone(),
        };
        let next_entity = inserted.entity;
        if let Some(tips) = tips {
            commands.spawn((
                Node {
                    width: percent(100),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                children![
                    text_bundle(
                        tips.0,
                        duration.0 / 2.0,
                        duration.0 - duration.0 / 2.0,
                        0.5,
                        duration.0 / 2.0,
                    ),
                    controls_bundle(
                        false,
                        image,
                        duration.0 / 2.0,
                        duration.0 - duration.0 / 2.0,
                        0.5,
                        duration.0 / 2.0,
                    )
                ],
            ));
        } else {
            commands.spawn(controls_bundle(
                true,
                image,
                duration.0 / 2.0,
                duration.0 - duration.0 / 2.0,
                0.5,
                duration.0 / 2.0,
            ));
        }
        commands.spawn((
            DespawnFinished,
            AnimationTarget(*camera),
            animations![
                (
                    Duration(duration.0),
                    Keyframe(CameraTransitionProgress(0.5)),
                ),
                Duration(0.5),
                system(move |mut commands: Commands| {
                    if let Some(prev) = prev_entity {
                        commands.entity(prev).despawn();
                    }
                    commands.entity(next_entity).insert(EnterMinigame);
                }),
                (
                    Duration(duration.0),
                    Keyframe(CameraTransitionProgress(1.0)),
                ),
            ],
        ));
    } else if cut.is_some() {
        let prev = runner.0;
        let next_entity = inserted.entity;
        let image = commands.spawn_empty().id();
        commands.spawn((
            DespawnFinished,
            animations![
                system(
                    move |mut commands: Commands,
                          sounds: Query<
                        Entity,
                        (With<SamplePlayer>, Or<(With<WinSfx>, With<LooseSfx>)>),
                    >,
                          assets: Res<MinigameAssets>,
                          mut rng: Single<&mut WyRand, With<GlobalRng>>,
                          mut fractal: Single<&mut Opacity, With<Fractal>>| {
                        fractal.0 = 0.0;
                        for entity in sounds.iter() {
                            commands.entity(entity).despawn();
                        }

                        if let Some(prev) = prev {
                            commands.entity(prev).despawn();
                        }

                        let texture = match rng.random_range(0..4) {
                            0 => assets.noise1.clone(),
                            1 => assets.noise2.clone(),
                            2 => assets.noise3.clone(),
                            3 => assets.noise4.clone(),
                            _ => unreachable!(),
                        };

                        let mut entity = commands.spawn((
                            ImageOf(image),
                            Node {
                                width: percent(100),
                                height: percent(100),
                                ..Default::default()
                            },
                            ImageNode::new(texture),
                        ));

                        if !no_cut_sfx {
                            entity.insert((
                                SamplePlayer::new(assets.cut.clone())
                                    .with_volume(Volume::Linear(0.8)),
                                PlaybackSettings::default().preserve(),
                            ));
                        }
                    }
                ),
                Duration(duration.0),
                system(move |mut commands: Commands| {
                    commands.entity(image).despawn();
                    commands.entity(next_entity).insert(EnterMinigame);
                }),
            ],
        ));
    } else {
        panic!("`RunTransition` target has not transition");
    }

    fn controls_bundle(
        abs: bool,
        image: Handle<Image>,
        delay: f32,
        dur_in: f32,
        pause: f32,
        dur_out: f32,
    ) -> impl Bundle {
        (
            ImageNode {
                image,
                color: Color::WHITE.with_alpha(0.0),
                ..Default::default()
            },
            Node {
                position_type: if abs {
                    PositionType::Absolute
                } else {
                    PositionType::Relative
                },
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                height: percent(25.0),
                ..Default::default()
            },
            ImageColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
            UiTranslationPx(Vec2::new(0.0, -20.0)),
            //
            AnimationTarget::entity(),
            DespawnFinished,
            animations![
                system(|mut commands: Commands, assets: Res<MinigameAssets>| {
                    commands.spawn((
                        SamplePlayer::new(assets.control.clone()).with_volume(Volume::Linear(0.6)),
                        PlaybackSettings::default().with_speed(0.5),
                        sample_effects![FreeverbNode::default()],
                    ));
                }),
                Duration(delay),
                (
                    Duration(dur_in),
                    Keyframe(ImageColor(Color::srgba(1.0, 1.0, 1.0, 1.0))),
                    Keyframe(UiTranslationPx(Vec2::ZERO)),
                    Easing::SineInOut,
                ),
                Duration(pause),
                (
                    Duration(dur_out),
                    Keyframe(ImageColor(Color::srgba(1.0, 1.0, 1.0, 0.0))),
                    Easing::SineInOut,
                ),
            ],
        )
    }

    fn text_bundle(text: &str, delay: f32, dur_in: f32, pause: f32, dur_out: f32) -> impl Bundle {
        (
            Text::new(text),
            TextFont::from_font_size(80.0),
            TextColorTween(Color::WHITE.with_alpha(0.0)),
            Node {
                // position_type: PositionType::Absolute,
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                ..Default::default()
            },
            //
            AnimationTarget::entity(),
            DespawnFinished,
            animations![
                Duration(delay),
                (
                    Duration(dur_in),
                    Keyframe(TextColorTween(Color::srgba(1.0, 1.0, 1.0, 1.0))),
                    Easing::SineInOut,
                ),
                Duration(pause),
                (
                    Duration(dur_out),
                    Keyframe(TextColorTween(Color::srgba(1.0, 1.0, 1.0, 0.0))),
                    Easing::SineInOut,
                ),
            ],
        )
    }
}

#[derive(Component)]
pub struct EnterMinigame;

#[derive(Component)]
pub struct ExitMinigame;

#[derive(Event)]
struct ExhaustedMinigames;

fn exhausted_minigames(
    _: On<ExhaustedMinigames>,
    mut commands: Commands,
    state: Res<State<GameState>>,
) {
    match state.get() {
        GameState::PhaseOne => {
            commands.run_system_cached(exit_phase_one);
        }
        _ => {
            commands.set_state(GameState::Outro);
        }
    }
}
