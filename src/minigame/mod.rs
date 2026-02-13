use crate::{
    animation::*,
    animations,
    camera::MovementSensitivity,
    fractal::{Fractal, ResetFractal},
    state::GameState,
    transition::{Transition, TransitionProgress},
};
use bevy::{
    color::palettes::css::RED,
    ecs::{entity_disabling::Disabled, system::SystemId},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use bevy_seedling::prelude::*;
use fever_macros::Lerp;
use rand::seq::IteratorRandom;

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
    .add_systems(OnEnter(Minigame::EnterWipe), wipe)
    .add_systems(OnExit(Minigame::Choose), available_after)
    .add_systems(OnEnter(Minigame::Success), success)
    .add_systems(OnEnter(Minigame::Failure), failure)
    .add_systems(
        Update,
        (
            tick_timer,
            (image_color, ui_translation).after(AnimationSystems::Interpolate),
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
    #[asset(path = "third-party/keyboard.png")]
    keyboard: Handle<Image>,
    #[asset(path = "third-party/ear.png")]
    ear: Handle<Image>,
    //
    #[asset(path = "sfx/control.ogg")]
    pub control: Handle<AudioSample>,
    #[asset(path = "sfx/timeout.ogg")]
    timeout: Handle<AudioSample>,
    #[asset(path = "sfx/succeed.ogg")]
    succeed: Handle<AudioSample>,
    //
    #[asset(path = "sfx/glyph.ogg")]
    pub glyph: Handle<AudioSample>,
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SubStates, Component)]
#[source(GameState = GameState::Playing)]
pub enum Minigame {
    #[default]
    None,
    Choose,
    EnterWipe,
    Wipe,
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
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut writer: MessageWriter<AppExit>,
) {
    let next = if RANDOM {
        available.iter().choose(&mut rng)
    } else {
        available.iter().next()
    };
    if let Some(entity) = next {
        commands.entity(entity).insert(Chosen);
        commands.set_state(Minigame::EnterWipe);
    } else {
        // TODO: finish screen
        // NOTE: changing this will cause the success and failure screens to not
        // be despawned!!!!!
        // panic!("finished");
        writer.write(AppExit::Success);
    }
}

#[derive(Component)]
pub enum Description {
    Keyboard,
    Wasd,
    Mouse,
    Ear,
}

fn wipe(
    mut commands: Commands,
    camera: Single<Entity, With<Transition>>,
    assets: Res<MinigameAssets>,
    chosen: Single<&Description, With<Chosen>>,
) {
    let duration = 1.0;
    let image = match *chosen {
        Description::Wasd => assets.wasd.clone(),
        Description::Mouse => assets.mouse.clone(),
        Description::Keyboard => assets.keyboard.clone(),
        Description::Ear => assets.ear.clone(),
    };
    commands.spawn(controls_bundle(
        image,
        duration / 2.0,
        duration - duration / 2.0,
        0.5,
        duration / 2.0,
    ));
    commands.entity(*camera).insert(TransitionProgress(0.0));
    commands.spawn((
        AnimationTarget(*camera),
        DespawnFinished,
        animations![
            (Duration(duration), Keyframe(TransitionProgress(0.5))),
            set_state(Minigame::Wipe),
            system(choose_minigame),
            Duration(0.5),
            (Duration(duration), Keyframe(TransitionProgress(1.0))),
        ],
    ));
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
        .insert(DespawnOnExit(Minigame::EnterWipe));
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
    commands.set_state(Minigame::Choose);
}

fn failure(mut commands: Commands, assets: Res<MinigameAssets>, mut failed: Local<usize>) {
    commands.spawn(SamplePlayer::new(assets.timeout.clone()).with_volume(Volume::Linear(1.5)));

    *failed += 1;
    if *failed >= 3 {
        *failed = 0;
        // TODO: very obvious restarting animation
        commands.set_state(Minigame::None);
        commands.set_state(GameState::Restart);
    } else {
        commands.set_state(Minigame::Choose);
    }
}

fn controls_bundle(
    image: Handle<Image>,
    delay: f32,
    dur_in: f32,
    pause: f32,
    dur_out: f32,
) -> impl Bundle {
    (
        ImageNode::new(image),
        Node {
            position_type: PositionType::Absolute,
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
        DespawnOnExit(GameState::Intro),
        animations![
            system(|mut commands: Commands, assets: Res<MinigameAssets>| {
                commands.spawn((
                    DespawnOnExit(GameState::Intro),
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
