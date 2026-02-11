use crate::{
    fractal::lock_camera,
    minigame::{
        AvailableAfter, Description, Minigame, MinigameRoot, NotRandom, OnVariationEnable,
        Variation, VariationSet,
    },
    state::GameState,
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<DreamAssets>())
        .add_observer(advance_text)
        .add_systems(OnEnter(GameState::Playing), init_targets)
        .add_systems(OnEnter(Minigame::Dream), lock_camera)
        .add_systems(
            Update,
            (await_input, empty).run_if(in_state(Minigame::Dream)),
        );
}

#[derive(AssetCollection, Resource)]
struct DreamAssets {
    #[asset(path = "sfx/glyph.ogg")]
    glyph: Handle<AudioSample>,
    //
    #[asset(path = "music/rain.ogg")]
    rain: Handle<AudioSample>,
    #[asset(path = "music/deep.ogg")]
    deep: Handle<AudioSample>,
    #[asset(path = "music/birds.ogg")]
    birds: Handle<AudioSample>,
    #[asset(path = "music/hell.ogg")]
    hell: Handle<AudioSample>,
}

#[derive(Component)]
struct Dream;

fn init_targets(mut commands: Commands, assets: Res<DreamAssets>) {
    let empty_start = OnVariationEnable(commands.register_system(
        |_: In<Entity>, mut commands: Commands| {
            commands.spawn((
                DespawnOnExit(Minigame::Dream),
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                GlobalZIndex(500),
            ));
        },
    ));
    let on_start = OnVariationEnable(commands.register_system(advance_text_with));

    commands.spawn((
        MinigameRoot,
        DespawnOnExit(GameState::Playing),
        AvailableAfter(6),
        Minigame::Dream,
        Dream,
        Description("LISTEN\n(SPACE/ENTER)"),
        children![(
            VariationSet,
            NotRandom,
            children![
                (
                    Variation,
                    empty_start,
                    Empty(Timer::from_seconds(5.0, TimerMode::Once)),
                    SamplePlayer::new(assets.deep.clone())
                        .with_volume(Volume::Linear(1.0))
                        .looping(),
                ),
                (
                    Variation,
                    TextSequence,
                    on_start,
                    SamplePlayer::new(assets.rain.clone())
                        .with_volume(Volume::Linear(1.0))
                        .looping(),
                    children![TextSeg(pretty!(
                        "|2|You|0.25| should not|0.5| be [here](shake, red)<0.5>..."
                    ))]
                ),
                (
                    Variation,
                    TextSequence,
                    on_start,
                    SamplePlayer::new(assets.birds.clone())
                        .with_volume(Volume::Linear(1.0))
                        .looping(),
                    children![
                        TextSeg(pretty!("|2|There is no end.")),
                        TextSeg(pretty!(
                            "A [dream](red) with no beginning has no end<0.5>..."
                        ))
                    ]
                ),
                (
                    Variation,
                    TextSequence,
                    on_start,
                    SamplePlayer::new(assets.hell.clone())
                        .with_volume(Volume::Linear(1.0))
                        .looping(),
                    children![
                        TextSeg(pretty!("What do you seek in this [dream](red)?")),
                        TextSeg(pretty!("This [dream](red) will only take from you.")),
                        (
                            TextSeg(pretty!("|1.0|How did you get [here](red)?|0.25|")),
                            SkipInput
                        ),
                    ]
                )
            ],
        )],
    ));
}

#[derive(Component)]
struct Empty(Timer);

fn empty(mut commands: Commands, mut empty: Single<&mut Empty>, time: Res<Time>) {
    empty.0.tick(time.delta());
    if empty.0.just_finished() {
        commands.set_state(Minigame::Success);
    }
}

#[derive(Component)]
#[require(TextIndex)]
struct TextSequence;

#[derive(Default, Component)]
struct TextIndex(usize);

#[derive(Component)]
struct TextSeg(ParsedPrettyText<Text>);

fn advance_text(
    finished: On<TypewriterFinished>,
    mut commands: Commands,
    skip: Query<(), With<SkipInput>>,
) {
    if skip.get(finished.entity).is_ok() {
        commands.run_system_cached_with(advance_text_with, Entity::PLACEHOLDER);
    } else {
        commands.spawn(AwaitInput);
    }
}

#[derive(Component)]
struct AwaitInput;

#[derive(Component)]
struct SkipInput;

fn await_input(
    mut commands: Commands,
    await_input: Query<Entity, With<AwaitInput>>,
    active: Query<Entity, With<Typewriter>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if await_input.is_empty() {
        if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
            for entity in active.iter() {
                commands.entity(entity).insert(FinishTypewriter);
            }
        }
        return;
    }
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        for entity in await_input.iter() {
            commands.entity(entity).despawn();
        }
        commands.run_system_cached_with(advance_text_with, Entity::PLACEHOLDER);
    }
}

fn advance_text_with(
    _: In<Entity>,
    mut commands: Commands,
    seq: Single<(&mut TextIndex, Option<&Children>), With<TextSequence>>,
    text: Query<(&TextSeg, Has<SkipInput>)>,
) {
    let (mut index, seq) = seq.into_inner();
    if let Some(children) = seq
        && let Some(child) = children.iter().nth(index.0)
    {
        let (text, skip_input) = text.get(child).unwrap();
        commands
            .spawn((
                DespawnOnExit(Minigame::Dream),
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                GlobalZIndex(500),
            ))
            .with_children(|s| {
                let mut entity = s.spawn((
                    Node {
                        width: percent(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    text.0.clone().into_bundle(),
                    TextFont::from_font_size(50.0),
                    TextLayout::new_with_justify(Justify::Center),
                    Typewriter::new(15.0),
                ));
                entity.observe(
                    |revealed: On<Revealed<Char>>,
                     mut commands: Commands,
                     assets: Res<DreamAssets>| {
                        if revealed.event().text != " " {
                            commands.spawn((
                                SamplePlayer::new(assets.glyph.clone())
                                    .with_volume(Volume::Linear(0.8)),
                                RandomPitch::new(0.05),
                            ));
                        }
                    },
                );
                if skip_input {
                    entity.insert(SkipInput);
                }
            });
        index.0 += 1;
        return;
    }
    commands.set_state(Minigame::Success);
}
