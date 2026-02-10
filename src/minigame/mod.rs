use crate::state::GameState;
use bevy::prelude::*;

mod count_down;
mod matching;

pub fn plugin(app: &mut App) {
    app.add_plugins((count_down::plugin, matching::plugin))
        .add_sub_state::<Minigame>()
        .add_systems(OnEnter(Minigame::None), start)
        .add_systems(OnEnter(Minigame::Choose), choose);
}

fn start(mut commands: Commands) {
    commands.set_state(Minigame::Countdown);
}

fn choose(mut commands: Commands) {
    commands.set_state(Minigame::Matching);
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, SubStates)]
#[source(GameState = GameState::Playing)]
pub enum Minigame {
    #[default]
    None,
    Countdown,
    Choose,
    //
    Matching,
}
