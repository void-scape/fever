// #[cfg(feature = "dev")]
// use crate::minigame::prelude::Transition;
use crate::prelude::*;
// #[cfg(feature = "dev")]
// use bevy::dev_tools::states::log_transitions;
use bevy::{color::palettes::css::RED, ecs::system::SystemId, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;

#[allow(unused)]
mod prelude {
    pub use super::choose::*;
    pub use super::dream::*;
    pub use super::matching::*;
    pub use super::path::*;
    pub use super::tempest::*;
    pub use super::transition::*;
    pub use super::typing::*;
}

mod choose;
mod dream;
mod matching;
mod path;
mod tempest;
mod transition;
mod typing;

pub fn minigame_plugin(app: &mut App) {
    // #[cfg(feature = "dev")]
    // app.add_systems(Update, log_transitions::<Minigame>)
    //     .add_systems(Update, log_transitions::<Transition>);

    app.add_plugins((
        transition::transition_plugin,
        choose::choose_plugin,
        //
        matching::matching_plugin,
        path::path_plugin,
        tempest::tempest_plugin,
        dream::dream_plugin,
        typing::typing_plugin,
    ))
    .add_sub_state::<Minigame>()
    .add_loading_state(LoadingState::new(GameState::Loading).load_collection::<MinigameAssets>())
    .add_observer(end_timer)
    .add_systems(OnEnter(Minigame::None), choose::run_choose_systems)
    .add_systems(Update, tick_timer);
}

#[derive(AssetCollection, Resource)]
pub struct MinigameAssets {
    #[asset(path = "third-party/wasd.png")]
    pub wasd: Handle<Image>,
    #[asset(path = "third-party/mouse.png")]
    pub mouse: Handle<Image>,
    #[asset(path = "third-party/keyboard.png")]
    keyboard: Handle<Image>,
    //
    #[asset(path = "sfx/control.ogg")]
    pub control: Handle<AudioSample>,
    #[asset(path = "sfx/timeout.ogg")]
    timeout: Handle<AudioSample>,
    #[asset(path = "sfx/succeed.ogg")]
    succeed: Handle<AudioSample>,
    //
    #[asset(path = "sfx/narrator-glyph.ogg")]
    pub narrator_glyph: Handle<AudioSample>,
    #[asset(path = "sfx/presence-glyph.ogg")]
    pub presence_glyph: Handle<AudioSample>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SubStates, Component)]
#[source(GameState = GameState::Playing)]
pub enum Minigame {
    #[default]
    None,
    Matching,
    Path,
    Tempest,
    Dream,
    Typing,
    //
    Reset,
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct MinigameRoot;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct VariationSet;

#[derive(Component)]
pub struct NotRandom;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Variation;

#[derive(Clone, Copy, Component)]
pub struct OnVariationEnable(pub SystemId<In<Entity>, ()>);

#[derive(Component)]
pub struct GameTimer(pub Timer);

#[derive(Clone, Copy, Component)]
pub struct TimerDuration(pub f32);

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
    commands.run_system_cached(failure);
}

pub fn success(mut commands: Commands, assets: Res<MinigameAssets>) {
    commands.spawn((
        SamplePlayer::new(assets.succeed.clone()).with_volume(Volume::Linear(0.4)),
        PlaybackSettings::default().with_speed(0.8),
        // sample_effects![FreeverbNode {
        //     room_size: 0.2,
        //     damping: 0.9,
        //     width: 0.1,
        //     ..Default::default()
        // }],
    ));
    commands.run_system_cached(choose::run_choose_systems);
}

pub fn failure(mut commands: Commands, assets: Res<MinigameAssets>, _failed: Local<usize>) {
    commands.spawn((
        SamplePlayer::new(assets.timeout.clone()).with_volume(Volume::Linear(0.4)),
        PlaybackSettings::default().with_speed(0.9),
        // sample_effects![FreeverbNode {
        //     room_size: 0.2,
        //     damping: 0.9,
        //     width: 0.1,
        //     ..Default::default()
        // }],
    ));

    // TODO: restart state, or not?
    // *failed += 1;
    // if *failed >= 3 {
    //     *failed = 0;
    //     // TODO: very obvious restarting animation
    //     commands.set_state(Minigame::None);
    //     commands.set_state(GameState::Restart);
    // } else {
    commands.run_system_cached(choose::run_choose_systems);
    // }
}
