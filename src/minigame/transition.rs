use crate::{minigame::prelude::*, prelude::*};
use bevy::prelude::*;
use bevy_seedling::prelude::*;

pub fn transition_plugin(app: &mut App) {
    app.init_state::<Transition>().add_systems(
        OnEnter(Transition::Enter),
        (controls_transition, cut_transition),
    );
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States, Component)]
pub enum Transition {
    #[default]
    None,
    Enter,
    Exit,
}

pub fn fade_volume_transition() -> impl Bundle {
    (
        LinearVolume(0.0),
        AnimationTarget::entity(),
        animations![
            fade_volume(1.0, 1.0),
            block_in_state(Transition::None),
            fade_volume(1.0, 0.0)
        ],
    )
}

#[derive(Component)]
pub enum ControlsTransition {
    Keyboard,
    Wasd,
    Mouse,
}

fn controls_transition(
    mut commands: Commands,
    camera: Single<Entity, With<CameraTransition>>,
    assets: Res<MinigameAssets>,
    state: Single<&Minigame, With<ChosenRoot>>,
    controls: Single<&ControlsTransition, With<ChosenVariation>>,
) {
    let duration = 1.0;
    let image = match *controls {
        ControlsTransition::Wasd => assets.wasd.clone(),
        ControlsTransition::Mouse => assets.mouse.clone(),
        ControlsTransition::Keyboard => assets.keyboard.clone(),
    };
    commands.spawn(controls_bundle(
        image,
        duration / 2.0,
        duration - duration / 2.0,
        0.5,
        duration / 2.0,
    ));
    commands
        .entity(*camera)
        .insert(CameraTransitionProgress(0.0));
    commands.spawn((
        AnimationTarget(*camera),
        DespawnFinished,
        animations![
            (Duration(duration), Keyframe(CameraTransitionProgress(0.5))),
            set_state(Minigame::Reset),
            set_state(**state),
            set_state(Transition::Exit),
            system(enable_variation),
            Duration(0.5),
            (Duration(duration), Keyframe(CameraTransitionProgress(1.0))),
            set_state(Transition::None),
        ],
    ));
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
        animations![
            system(|mut commands: Commands, assets: Res<MinigameAssets>| {
                commands.spawn((
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

#[derive(Component)]
pub struct CutTransition {
    pub image: Handle<Image>,
    pub sfx: Handle<AudioSample>,
}

fn cut_transition(
    mut commands: Commands,
    state: Single<&Minigame, With<ChosenRoot>>,
    _: Single<(), (With<CutTransition>, With<ChosenVariation>)>,
) {
    let entity = commands.spawn_empty().id();
    commands.spawn((
        DespawnFinished,
        animations![
            set_state(Minigame::Reset),
            system(
                move |mut commands: Commands,
                      sounds: Query<Entity, With<SamplePlayer>>,
                      cut: Single<&CutTransition, With<ChosenVariation>>| {
                    for entity in sounds.iter() {
                        // TODO: find a better way to get rid of the success/failure sound
                        commands.entity(entity).despawn();
                    }

                    commands.entity(entity).insert((
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..Default::default()
                        },
                        ImageNode::new(cut.image.clone()),
                        SamplePlayer::new(cut.sfx.clone()).with_volume(Volume::Linear(0.8)),
                        PlaybackSettings::default().preserve(),
                    ));
                }
            ),
            Duration(0.15),
            set_state(**state),
            set_state(Transition::Exit),
            system(enable_variation),
            system(move |mut commands: Commands| {
                commands.entity(entity).despawn();
            }),
            set_state(Transition::None),
        ],
    ));
}
