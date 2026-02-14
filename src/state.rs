use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub fn state_plugin(app: &mut App) {
    #[cfg(not(feature = "explore"))]
    let state = GameState::Intro;
    #[cfg(feature = "explore")]
    let state = GameState::Explore;
    app.init_state::<GameState>()
        .add_loading_state(LoadingState::new(GameState::Loading).continue_to_state(state))
        .add_systems(OnEnter(GameState::Restart), restart);
}

#[allow(unused)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Intro,
    Playing,
    Restart,
    Outro,
    #[cfg(feature = "dev")]
    Explore,
}

fn restart(mut commands: Commands) {
    commands.set_state(GameState::Playing);
}
