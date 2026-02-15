use crate::prelude::*;
use bevy::prelude::*;
use bevy_pretty_text::prelude::*;

pub fn restart_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Restart(&GameState::PhaseOne)), restart)
        .add_systems(OnEnter(GameState::Restart(&GameState::PhaseTwo)), restart);
}

fn restart(
    mut commands: Commands,
    mut palette: ResMut<TransitionPalette>,
    camera: Single<Entity, With<Camera>>,
) {
    let tdur = 0.15;
    *palette = TransitionPalette::Red;
    commands.spawn((
        DespawnFinished,
        AnimationTarget(*camera),
        animations![
            (Duration(2.0), Keyframe(CameraTransitionProgress(0.5))),
            Duration(1.0),
            system(narrator_glyph),
            await_finish(pretty!("I will awake from this dream lest I-|0.1|")),
            system(move |mut commands: Commands| {
                commands
                    .spawn((CutTransition, TransitionDuration(tdur)))
                    .insert(RunTransition(None));
            }),
            Duration(tdur),
            system(restart_state),
        ],
    ));
}

fn restart_state(mut commands: Commands, state: Res<State<GameState>>) {
    match *state.get() {
        GameState::Restart(&GameState::PhaseOne) => {
            commands.set_state(GameState::PhaseOne);
        }
        GameState::Restart(&GameState::PhaseTwo) => {
            commands.set_state(GameState::PhaseTwo);
        }
        state => {
            panic!("cannot reset in {:?}", state);
        }
    }
}
