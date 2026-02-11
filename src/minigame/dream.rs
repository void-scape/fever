use crate::{
    fractal::lock_camera,
    minigame::{
        Description, Minigame, MinigameRoot, NotRandom, OnVariationEnable, Variation, VariationSet,
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
    #[asset(path = "music/rain.wav")]
    rain: Handle<AudioSample>,
    #[asset(path = "music/deep.wav")]
    deep: Handle<AudioSample>,
    #[asset(path = "music/birds.wav")]
    birds: Handle<AudioSample>,
    #[asset(path = "music/hell.wav")]
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
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
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
                    TextIndex(0),
                    on_start,
                    SamplePlayer::new(assets.rain.clone())
                        .with_volume(Volume::Linear(1.0))
                        .looping(),
                    children![TextSeg(pretty!(
                        "|2|You|0.25| should not|0.5| be [here](shake)|0.1|..."
                    ))]
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
        commands.set_state(Minigame::Failure);
    }
}

#[derive(Component)]
struct TextSequence;

#[derive(Component)]
struct TextSeg(ParsedPrettyText<Text>);

#[derive(Component)]
struct TextIndex(usize);

fn advance_text(_: On<TypewriterFinished>, mut commands: Commands) {
    commands.spawn(AwaitInput);
}

#[derive(Component)]
struct AwaitInput;

fn await_input(
    mut commands: Commands,
    await_input: Single<Entity, With<AwaitInput>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        commands.entity(*await_input).despawn();
        commands.run_system_cached_with(advance_text_with, Entity::PLACEHOLDER);
    }
}

fn advance_text_with(
    _: In<Entity>,
    mut commands: Commands,
    seq: Single<(&mut TextIndex, Option<&Children>), With<TextSequence>>,
    text: Query<&TextSeg>,
) {
    let (mut index, seq) = seq.into_inner();
    if let Some(children) = seq
        && let Some(child) = children.iter().nth(index.0)
    {
        let text = text.get(child).unwrap();
        commands
            .spawn((
                DespawnOnExit(Minigame::Dream),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                GlobalZIndex(500),
            ))
            .with_children(|s| {
                s.spawn((
                    text.0.clone().into_bundle(),
                    TextFont::from_font_size(50.0),
                    TextLayout::new(Justify::Center, LineBreak::NoWrap),
                    Typewriter::new(15.0),
                ))
                .observe(
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
            });
        index.0 += 1;
        return;
    }
    commands.set_state(Minigame::Failure);
}
