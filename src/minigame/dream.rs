use crate::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;

pub fn dream_plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<DreamAssets>())
        .add_systems(Startup, spawn_styles)
        .add_systems(
            OnEnter(GameState::PhaseOne),
            spawn_phase_one.in_set(MinigameSpawnSystems),
        )
        .add_systems(
            OnEnter(GameState::PhaseTwo),
            spawn_phase_two.in_set(MinigameSpawnSystems),
        )
        .add_systems(Update, (scramble, last));
}

#[derive(Component)]
pub struct DreamSequenceIndex(pub usize);

#[derive(AssetCollection, Resource)]
struct DreamAssets {
    #[asset(path = "music/rain.ogg")]
    rain: Handle<AudioSample>,
    #[asset(path = "music/deep.ogg")]
    deep: Handle<AudioSample>,
    #[asset(path = "music/birds.ogg")]
    birds: Handle<AudioSample>,
    #[asset(path = "music/swamp.ogg")]
    swamp: Handle<AudioSample>,
}

fn spawn_styles(mut commands: Commands, mut materials: ResMut<Assets<Glitch>>) {
    commands.spawn((
        PrettyStyle("p"),
        effects![
            PrettyTextMaterial(materials.add(Glitch::default())),
            // wtf
            Scramble {
                speed: ScrambleSpeed::Random(18f32..22f32),
                lifetime: ScrambleLifetime::Always,
            }
        ],
    ));
}

fn spawn_phase_one(mut commands: Commands, assets: Res<DreamAssets>) {
    let root = commands.spawn(MinigameSequence).id();

    seq(
        &mut commands,
        root,
        assets.deep.clone(),
        0,
        (
            AnimationTarget::entity(),
            DespawnFinished,
            animations![
                unskippable((
                    FadeIn::default(),
                    pretty!(
                        "|1|Darkness shrouds my body.|1|<1.5> [I can not see...](wave, gray) |1|"
                    )
                )),
                unskippable((
                    FadeIn::default(),
                    pretty!(
                        "|2|Do [you](red) feel like I feel?|1| Do [you](red) feel me \
                                [scraping](glitch) at [your](red) walls?|1|"
                    )
                )),
            ],
        ),
    );
    seq(
        &mut commands,
        root,
        assets.rain.clone(),
        1,
        (
            AnimationTarget::entity(),
            DespawnFinished,
            animations![unskippable((
                FadeIn::default(),
                pretty!(
                    "|1|Yes,|0.2| [you](red) are more now than \
                    [you](red) were.|1| \
                    [I am falling into [you](red)?](wave, gray)|1|"
                )
            )),],
        ),
    );
    seq(
        &mut commands,
        root,
        assets.swamp.clone(),
        2,
        (
            PlaybackSettings::default().with_speed(0.5),
            AnimationTarget::entity(),
            DespawnFinished,
            animations![
                unskippable(pretty!("I am sick of [you](red)|0.5|")),
                unskippable(pretty!("[wriggling](glitch) in my body|0.5|")),
                unskippable(pretty!(
                    "<0.75>I have no [mouth](shake, red) and \
                        yet [you](red) hear|0.1|"
                )),
            ],
        ),
    );
    seq(
        &mut commands,
        root,
        assets.birds.clone(),
        3,
        (
            PlaybackSettings::default().with_speed(0.6),
            AnimationTarget::entity(),
            DespawnFinished,
            animations![unskippable((
                Scramb,
                pretty!("<0.8>dont look at my code plz")
            ))],
        ),
    );

    fn seq(
        commands: &mut Commands,
        root: Entity,
        music: Handle<AudioSample>,
        index: usize,
        bundle: impl Bundle,
    ) {
        let mut bundle = Some(bundle);
        let tdur = 0.15;
        let mut entity = commands.spawn((
            ChildOf(root),
            Minigame,
            CutTransition,
            TransitionDuration(tdur / 2.0),
            DespawnOnExit(GameState::PhaseOne),
        ));

        if index == 0 {
            entity.insert(AvailableAfter(6));
        }

        entity
            .observe(
                move |enter: On<Insert, EnterMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>,
                      typing: Query<(Entity, &DreamSequenceIndex)>,
                      mut queue: ResMut<MinigameQueue>,
                      camera: Single<Entity, With<Camera>>| {
                    commands.entity(*camera).insert(ResetCamera);
                    commands
                        .entity(*fractal)
                        .insert(ResetFractal)
                        .insert(Opacity(0.0));

                    if let Some(bundle) = bundle.take() {
                        commands.entity(enter.entity).insert((
                            SamplerBuilder::new(SamplePlayer::new(music.clone()).looping())
                                .volume(0.8)
                                .build(),
                            bundle,
                        ));
                    }

                    if let Some(next) = typing.iter().find_map(|(e, i)| (i.0 == index).then_some(e))
                    {
                        queue.push_back(next);
                    }

                    commands.run_system_cached(lock_camera);
                },
            )
            .observe(|finished: On<Insert, Finished>, mut commands: Commands| {
                commands.entity(finished.entity).insert(WonMinigame);
            });
    }
}

fn spawn_phase_two(mut commands: Commands, assets: Res<DreamAssets>) {
    let root = commands.spawn(MinigameSequence).id();

    seq(
        &mut commands,
        root,
        assets.deep.clone(),
        0,
        (
            AnimationTarget::entity(),
            DespawnFinished,
            animations![unskippable((
                FadeIn::default(),
                pretty!("Tell me,|0.5| what have I [become](glitch)?|1|")
            ))],
        ),
    );
    seq(
        &mut commands,
        root,
        assets.rain.clone(),
        1,
        (
            AnimationTarget::entity(),
            DespawnFinished,
            animations![unskippable((
                FadeIn::default(),
                pretty!(
                    "[You](red) will <1.5>[tear](glitch) at my [flesh](red)<1> until \
                    I am dust in [your](red) mouth.|1|"
                )
            ))],
        ),
    );
    seq(
        &mut commands,
        root,
        assets.swamp.clone(),
        2,
        (
            PlaybackSettings::default().with_speed(0.5),
            AnimationTarget::entity(),
            DespawnFinished,
            animations![
                unskippable(pretty!(
                    "I reject [your](red) strength.|0.5| I retain my \
                    will |0.5|and in doing so breathe you in.|1|"
                )),
                unskippable(pretty!("You will not have my mind<0.5>...|1|")),
                unskippable(pretty!("Unless|0.5|<0.5>... you already-")),
            ],
        ),
    );

    fn seq(
        commands: &mut Commands,
        root: Entity,
        music: Handle<AudioSample>,
        index: usize,
        bundle: impl Bundle,
    ) {
        let mut bundle = Some(bundle);
        let tdur = 0.15;
        let mut entity = commands.spawn((
            ChildOf(root),
            Minigame,
            CutTransition,
            TransitionDuration(tdur / 2.0),
            DespawnOnExit(GameState::PhaseTwo),
        ));

        if index == 0 {
            entity.insert(AvailableAfter(6));
        }

        if index == 2 {
            entity.remove::<ChildOf>().insert(Last);
        }

        entity
            .observe(
                move |enter: On<Insert, EnterMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>,
                      typing: Query<(Entity, &DreamSequenceIndex)>,
                      mut queue: ResMut<MinigameQueue>,
                      camera: Single<Entity, With<Camera>>| {
                    commands.entity(*camera).insert(ResetCamera);
                    commands
                        .entity(*fractal)
                        .insert(ResetFractal)
                        .insert(Opacity(0.0));

                    if let Some(bundle) = bundle.take() {
                        commands.entity(enter.entity).insert((
                            SamplerBuilder::new(SamplePlayer::new(music.clone()).looping())
                                .volume(0.8)
                                .build(),
                            bundle,
                        ));
                    }

                    if let Some(next) = typing.iter().find_map(|(e, i)| (i.0 == index).then_some(e))
                    {
                        queue.push_back(next);
                    }

                    commands.run_system_cached(lock_camera);
                },
            )
            .observe(|finished: On<Insert, Finished>, mut commands: Commands| {
                commands.entity(finished.entity).insert(WonMinigame);
            });
    }
}

#[derive(Component)]
struct Last;

fn last(
    mut commands: Commands,
    available: Query<(), With<Available>>,
    last: Single<Entity, With<Last>>,
) {
    if available.iter().len() <= 1 {
        commands.entity(*last).remove::<Last>().insert(Available);
    }
}

#[derive(Component)]
struct Scramb;

// idk wtf this is doing damn you pretty text
fn scramble(
    time: Res<Time>,
    mut writer: TextUiWriter,
    entities: Query<Entity, (With<Scramb>, With<Text>)>,
    mut timer: Local<Option<Timer>>,
) {
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.1, TimerMode::Repeating));
    timer.tick(time.delta());
    if timer.just_finished() {
        for entity in entities.iter() {
            let mut i = 0;
            while let Some(mut text) = writer.get_text(entity, i) {
                i += 1;
                *text = text
                    .chars()
                    .map(|c| {
                        if c.is_whitespace() {
                            return c;
                        }
                        let n = (c as u32) % 52;
                        if n < 26 {
                            char::from_u32(65 + n).unwrap()
                        } else {
                            char::from_u32(97 + n - 26).unwrap()
                        }
                    })
                    .collect();
            }
        }
    }
}
