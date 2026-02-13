use crate::{
    animation::AnimationSystems,
    camera::MovementSensitivity,
    fractal::{Fractal, ResetFractal},
    state::GameState,
};
use bevy::{
    color::palettes::{css::RED, tailwind::GREEN_800},
    ecs::{entity_disabling::Disabled, system::SystemId},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use bevy_seedling::{
    prelude::Volume,
    sample::{AudioSample, SamplePlayer},
};
use fever_macros::Lerp;
use rand::seq::IteratorRandom;

#[cfg(feature = "dev")]
const FAST: bool = false;
#[cfg(not(feature = "dev"))]
const FAST: bool = false;

#[cfg(feature = "dev")]
const RANDOM: bool = true;
#[cfg(not(feature = "dev"))]
const RANDOM: bool = true;

mod dream;
mod matching;
mod path;
mod tempest;
mod typing;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        //,
        matching::plugin,
        path::plugin,
        tempest::plugin,
        dream::plugin,
        typing::plugin,
    ))
    .add_sub_state::<Minigame>()
    .add_loading_state(LoadingState::new(GameState::Loading).load_collection::<MinigameAssets>())
    .add_observer(end_timer)
    .add_systems(OnEnter(Minigame::None), start)
    .add_systems(OnEnter(Minigame::Choose), choose)
    .add_systems(OnExit(Minigame::Choose), available_after)
    .add_systems(OnEnter(Minigame::Countdown), start_count_down)
    .add_systems(OnEnter(Minigame::Description), description)
    .add_systems(OnEnter(Minigame::Success), success)
    .add_systems(OnEnter(Minigame::Failure), failure)
    .add_systems(
        Update,
        (
            tick_timer,
            tick_text_transition,
            (image_color, ui_translation).after(AnimationSystems::Interpolate),
            count_down.run_if(in_state(Minigame::Countdown)),
            (clean_variation_sets, clean_minigame_roots).chain(),
        ),
    );
}

#[derive(AssetCollection, Resource)]
pub struct MinigameAssets {
    #[asset(path = "third-party/wasd.png")]
    pub wasd: Handle<Image>,
    #[asset(path = "third-party/mouse.png")]
    pub mouse: Handle<Image>,
    //
    #[asset(path = "sfx/timeout.ogg")]
    timeout: Handle<AudioSample>,
    #[asset(path = "sfx/succeed.ogg")]
    succeed: Handle<AudioSample>,
    //
    #[asset(path = "sfx/glyph.ogg")]
    pub glyph: Handle<AudioSample>,
    //
    #[asset(path = "sfx/boom.ogg")]
    boom: Handle<AudioSample>,
    #[asset(path = "sfx/start.ogg")]
    start: Handle<AudioSample>,
    #[asset(path = "images/count-down/0.png")]
    one: Handle<Image>,
    #[asset(path = "images/count-down/1.png")]
    two: Handle<Image>,
    #[asset(path = "images/count-down/2.png")]
    three: Handle<Image>,
}

#[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct ImageColor(pub Color);

fn image_color(mut nodes: Query<(&mut ImageNode, &ImageColor), Changed<ImageColor>>) {
    for (mut node, color) in nodes.iter_mut() {
        node.color = color.0;
    }
}

#[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct UiTranslationPx(pub Vec2);

fn ui_translation(
    mut nodes: Query<(&mut UiTransform, &UiTranslationPx), Changed<UiTranslationPx>>,
) {
    for (mut node, t) in nodes.iter_mut() {
        node.translation = Val2::px(t.x, t.y);
    }
}

#[derive(Component)]
struct TextTransition(Timer);

impl TextTransition {
    fn new(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Once))
    }
}

#[derive(Component)]
struct TextTransitionExit(SystemId);

fn tick_text_transition(
    mut commands: Commands,
    time: Res<Time>,
    timer: Single<(&mut TextTransition, &TextTransitionExit)>,
) {
    let (mut timer, exit) = timer.into_inner();
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        commands.run_system(exit.0);
    }
}

fn text_transition<M>(
    commands: &mut Commands,
    despawn_on_enter: Option<Minigame>,
    despawn_on_exit: Option<Minigame>,
    duration: f32,
    exit: impl IntoSystem<(), (), M> + 'static,
    text: impl Bundle,
) {
    let exit = commands.register_system(exit);
    let mut tt = commands.spawn((
        TextTransition::new(duration),
        TextTransitionExit(exit),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(500),
        children![(
            text,
            TextFont::from_font_size(100.0),
            TextLayout::new_with_justify(Justify::Center),
        )],
    ));
    debug_assert!(despawn_on_exit.is_some() || despawn_on_enter.is_some());
    if let Some(state) = despawn_on_enter {
        tt.insert(DespawnOnEnter(state));
    }
    if let Some(state) = despawn_on_exit {
        tt.insert(DespawnOnExit(state));
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SubStates, Component)]
#[source(GameState = GameState::Playing)]
pub enum Minigame {
    #[default]
    None,
    Choose,
    Countdown,
    Description,
    //
    Matching,
    Path,
    Tempest,
    Dream,
    Typing,
    //
    Success,
    Failure,
}

fn start(mut commands: Commands) {
    commands.set_state(Minigame::Choose);
}

#[derive(Component)]
struct Chosen;

fn choose(
    mut commands: Commands,
    available: Query<Entity, (With<MinigameRoot>, Without<AvailableAfter>)>,
    mut camera: Single<&mut Transform, With<Camera>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut writer: MessageWriter<AppExit>,
) {
    camera.translation = Vec3::ZERO;
    let next = if RANDOM {
        available.iter().choose(&mut rng)
    } else {
        available.iter().next()
    };
    if let Some(entity) = next {
        commands.entity(entity).insert(Chosen);
        if FAST {
            commands.run_system_cached(choose_minigame);
        } else {
            commands.set_state(Minigame::Countdown);
        }
    } else {
        // TODO: finish screen
        // NOTE: changing this will cause the success and failure screens to not
        // be despawned!!!!!
        // panic!("finished");
        writer.write(AppExit::Success);
    }
}

#[derive(Component)]
struct CountDown {
    timer: Timer,
    index: usize,
}

const CDV: f32 = 0.6;

fn start_count_down(mut commands: Commands, assets: Res<MinigameAssets>) {
    commands.spawn(SamplePlayer::new(assets.boom.clone()).with_volume(Volume::Linear(CDV)));
    commands.spawn((
        DespawnOnExit(Minigame::Countdown),
        CountDown {
            timer: Timer::from_seconds(0.33, TimerMode::Repeating),
            index: 0,
        },
        ImageNode::new(assets.three.clone()),
        ZIndex(10_000),
        Node {
            position_type: PositionType::Absolute,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            height: percent(100.0),
            ..Default::default()
        },
    ));
}

fn count_down(
    mut commands: Commands,
    time: Res<Time>,
    count_down: Single<(&mut CountDown, &mut ImageNode)>,
    assets: Res<MinigameAssets>,
) {
    let (mut cd, mut sprite) = count_down.into_inner();
    cd.timer.tick(time.delta());
    if cd.timer.just_finished() {
        cd.index += 1;
        match cd.index {
            1 => {
                commands
                    .spawn(SamplePlayer::new(assets.boom.clone()).with_volume(Volume::Linear(CDV)));
                sprite.image = assets.two.clone();
            }
            2 => {
                commands
                    .spawn(SamplePlayer::new(assets.boom.clone()).with_volume(Volume::Linear(CDV)));
                sprite.image = assets.one.clone();
            }
            _ => {
                commands.spawn(
                    SamplePlayer::new(assets.start.clone()).with_volume(Volume::Linear(CDV)),
                );
                commands.set_state(Minigame::Description);
            }
        }
    }
}

#[derive(Component)]
pub struct Description(pub &'static str);

fn description(mut commands: Commands, chosen: Single<&Description, With<Chosen>>) {
    // TODO: art
    text_transition(
        &mut commands,
        None,
        Some(Minigame::Description),
        1.0,
        choose_minigame,
        Text::new(chosen.0),
    );
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct MinigameRoot;

#[derive(Component)]
pub struct AvailableAfter(pub usize);

fn available_after(mut commands: Commands, mut available: Query<(Entity, &mut AvailableAfter)>) {
    for (entity, mut after) in available.iter_mut() {
        after.0 = after.0.saturating_sub(1);
        if after.0 == 0 {
            commands.entity(entity).remove::<AvailableAfter>();
        }
    }
}

fn choose_minigame(
    mut commands: Commands,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    root: Single<(Entity, &Children, &Minigame), (With<MinigameRoot>, With<Chosen>)>,
    sets: Query<(&Children, Has<NotRandom>), With<VariationSet>>,
    variations: Query<
        (Entity, Option<&StartTimer>, Option<&OnVariationEnable>),
        (With<Variation>, Allow<Disabled>),
    >,
    fractal: Single<Entity, With<Fractal>>,
    camera: Single<Entity, With<Camera>>,
) {
    commands.entity(*fractal).insert(ResetFractal);
    commands
        .entity(*camera)
        .insert(MovementSensitivity::default());

    let (entity, children, state) = root.into_inner();
    commands.entity(entity).remove::<Chosen>();
    commands.set_state(*state);

    let (set, not_random) = sets.iter_many(children).next().unwrap();
    let (entity, timer, on_enable) = if RANDOM && !not_random {
        variations.iter_many(set).choose(&mut rng).unwrap()
    } else {
        variations.iter_many(set).next().unwrap()
    };
    commands
        .entity(entity)
        .remove_recursive::<Children, Disabled>()
        .insert(DespawnOnExit(*state));
    if let Some(on_enable) = on_enable {
        commands.run_system_with(on_enable.0, entity);
    }
    if let Some(timer) = timer {
        commands.spawn((
            DespawnOnExit(*state),
            GameTimer(Timer::from_seconds(timer.0, TimerMode::Once)),
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

fn clean_minigame_roots(
    mut commands: Commands,
    roots: Query<Entity, (With<MinigameRoot>, Without<Children>)>,
) {
    for entity in roots.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct VariationSet;

#[derive(Component)]
pub struct NotRandom;

fn clean_variation_sets(
    mut commands: Commands,
    sets: Query<Entity, (With<VariationSet>, Without<Children>)>,
) {
    for entity in sets.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
#[require(Disabled, Transform, Visibility)]
pub struct Variation;

#[derive(Clone, Copy, Component)]
pub struct OnVariationEnable(pub SystemId<In<Entity>, ()>);

#[derive(Component)]
struct GameTimer(Timer);

#[derive(Clone, Copy, Component)]
pub struct StartTimer(pub f32);

fn tick_timer(
    mut commands: Commands,
    time: Res<Time>,
    timer: Single<(&mut GameTimer, &mut Text, &mut TextColor)>,
) {
    let (mut timer, mut text, mut color) = timer.into_inner();
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        commands.trigger(EndTimer);
    }

    text.0 = format!("{:.1}", timer.0.remaining_secs());
    if timer.0.remaining_secs() <= 2.0 {
        color.0 = RED.into()
    };
}

#[derive(Event)]
struct EndTimer;

fn end_timer(_: On<EndTimer>, mut commands: Commands) {
    commands.set_state(Minigame::Failure);
}

fn success(mut commands: Commands, assets: Res<MinigameAssets>) {
    commands.spawn(SamplePlayer::new(assets.succeed.clone()).with_volume(Volume::Linear(0.8)));

    if FAST {
        commands.set_state(Minigame::Choose);
        return;
    }

    // TODO: art
    text_transition(
        &mut commands,
        // TODO: This will not despawn is all the minigames are exhausted
        Some(Minigame::Countdown),
        None,
        1.0,
        exit,
        (Text::new("SUCCESS"), TextColor(GREEN_800.into())),
    );
    fn exit(mut commands: Commands) {
        commands.set_state(Minigame::Choose);
    }
}

fn failure(mut commands: Commands, assets: Res<MinigameAssets>, mut failed: Local<usize>) {
    commands.spawn(SamplePlayer::new(assets.timeout.clone()).with_volume(Volume::Linear(1.5)));

    if FAST {
        commands.set_state(Minigame::Choose);
        return;
    }

    *failed += 1;
    if *failed >= 3 {
        *failed = 0;
        // TODO: art
        text_transition(
            &mut commands,
            // TODO: This will not despawn is all the minigames are exhausted
            Some(Minigame::Countdown),
            None,
            1.0,
            restart,
            (Text::new("GAME OVER"), TextColor(RED.into())),
        );
    } else {
        // TODO: art
        text_transition(
            &mut commands,
            // TODO: This will not despawn is all the minigames are exhausted
            Some(Minigame::Countdown),
            None,
            1.0,
            exit,
            (Text::new("FAILURE"), TextColor(RED.into())),
        );
    };

    fn restart(mut commands: Commands) {
        commands.set_state(Minigame::None);
        commands.set_state(GameState::Restart);
    }
    fn exit(mut commands: Commands) {
        commands.set_state(Minigame::Choose);
    }
}
