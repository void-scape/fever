use crate::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;

pub fn state_plugin(app: &mut App) {
    #[cfg(not(feature = "explore"))]
    let state = GameState::Intro;
    #[cfg(feature = "explore")]
    let state = GameState::Explore;
    app.init_state::<GameState>()
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

pub fn exit_phase_one(
    mut commands: Commands,
    mut palette: ResMut<TransitionPalette>,
    camera: Single<Entity, With<Camera>>,
    assets: Res<MinigameAssets>,
) {
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
                (
                    AnimationTarget(*camera),
                    Duration(2.0),
                    Keyframe(CameraTransitionProgress(0.5))
                ),
                fade_volume(8.0, 0.5),
            ],
            system(narrator_glyph),
            await_input(pretty!(
                "Awake.|0.25| Falling<0.5>...|0.5|<1> Still falling,|0.25| still awake?|1|"
            )),
            await_input(pretty!("[Your](red) presence grows within and without me.")),
            await_input(pretty!("If there is an end,|1| I draw nearer<0.5>...")),
            system(move |mut commands: Commands| {
                commands.set_state(GameState::PhaseTwo);
            }),
            fade_volume(1.0, 0.0),
        ],
    ));
}
