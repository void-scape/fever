use crate::{minigame::dream::DreamSequenceIndex, prelude::*};
use bevy::{
    color::palettes::css::RED,
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;

pub fn typing_plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<TypingAssets>())
        .add_systems(
            OnEnter(GameState::PhaseOne),
            spawn_phase_one.in_set(MinigameSpawnSystems),
        )
        .add_systems(
            OnEnter(GameState::PhaseTwo),
            spawn_phase_one.in_set(MinigameSpawnSystems),
        )
        .add_systems(Update, (typing, animate_julia));
}

#[derive(AssetCollection, Resource)]
struct TypingAssets {
    #[asset(path = "images/fractals/star-ship.png")]
    star_ship: Handle<Image>,
    #[asset(path = "images/fractals/odd-julia.png")]
    odd_julia: Handle<Image>,
    #[asset(path = "images/fractals/pl-julia.png")]
    pl_julia: Handle<Image>,
    //
    #[asset(path = "sfx/unhovered.ogg")]
    incorrect: Handle<AudioSample>,
}

fn spawn_phase_one(mut commands: Commands, assets: Res<TypingAssets>) {
    text(
        &mut commands,
        "FEEL YOU",
        5.0,
        0,
        (
            Iterations(8.0),
            FractalTexture(assets.star_ship.clone()),
            BurningShip(1),
            CPlane(Vec2::new(0.4888928, 0.08791673)),
        ),
    );
    text(
        &mut commands,
        "INFINITELY FAR",
        7.0,
        1,
        (
            Iterations(8.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
    );
    text(
        &mut commands,
        "NO SEPARATION BETWEEN",
        10.0,
        2,
        (
            Iterations(4.0),
            FractalTexture(assets.odd_julia.clone()),
            BurningShip(1),
            CPlane(Vec2::new(-0.137, -1.257)),
        ),
    );
    text(
        &mut commands,
        "SUCCUMBING PRESENCE",
        9.0,
        3,
        (
            Iterations(4.0),
            FractalTexture(assets.odd_julia.clone()),
            Exponent(5.0),
            CPlane(Vec2::new(0.667, 0.512)),
        ),
    );

    fn text(
        commands: &mut Commands,
        text: impl Into<String>,
        time: f32,
        index: usize,
        bundle: impl Bundle,
    ) {
        let mut bundle = Some(bundle);
        let text = text.into();
        let tdur = 2.0;
        commands
            .spawn((
                Minigame,
                WinSfx,
                LooseSfx,
                DreamSequenceIndex(index),
                MinigameTimer::duration(time),
                ControlsTransition::Keyboard,
                TransitionDuration(tdur / 2.0),
                DespawnOnExit(GameState::PhaseOne),
            ))
            .observe(
                move |enter: On<Insert, EnterMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>| {
                    if let Some(bundle) = bundle.take() {
                        commands
                            .entity(*fractal)
                            .insert(ResetFractal)
                            .insert(bundle);
                    }

                    commands.entity(enter.entity).insert(Active).with_child((
                        Active,
                        TypingTarget {
                            text: text.clone(),
                            index: 0,
                        },
                        Text2d::default(),
                        children![
                            (
                                TextSpan::default(),
                                TextColor(RED.into()),
                                TextFont::from_font_size(80.0)
                            ),
                            (TextSpan::new(text.clone()), TextFont::from_font_size(80.0)),
                        ],
                    ));

                    commands.run_system_cached(lock_camera);
                    commands.run_system_cached(force_camera_origin);
                },
            )
            .observe(
                move |exit: On<Insert, ExitMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>| {
                    commands
                        .entity(exit.entity)
                        .remove::<AnimationComponents>()
                        .insert(animations![(
                            AnimationTarget(*fractal),
                            Duration(tdur / 1.5),
                            Keyframe(Exponent(12.0)),
                            Easing::ExponentialInOut
                        )]);
                    commands.run_system_cached(unforce_camera_origin);
                },
            );
    }
}

fn animate_julia(
    time: Res<Time>,
    mut cplane: Single<&mut CPlane, With<Fractal>>,
    _: Single<(), (With<Active>, With<TypingTarget>)>,
) {
    cplane.0 += Vec2::from_angle(time.elapsed_secs_wrapped()) * 0.0001;
}

#[derive(Component)]
struct Active;

#[derive(Component)]
struct TypingTarget {
    text: String,
    index: usize,
}

fn typing(
    mut commands: Commands,
    mut input: MessageReader<KeyboardInput>,
    entity: Single<Entity, (With<Active>, Without<ExitMinigame>, With<Minigame>)>,
    target: Single<(&mut TypingTarget, &Children), With<Active>>,
    mut sections: Query<&mut TextSpan>,
    assets: Res<TypingAssets>,
    text_assets: Res<MinigameAssets>,
) {
    let (mut typing, ui) = target.into_inner();
    let chars = typing.text.chars().collect::<Vec<_>>();
    for event in input.read() {
        if event.state.is_pressed()
            && let Key::Character(s) = &event.logical_key
            && let Some(c) = s.chars().next()
            && typing.index < chars.len()
        {
            let expected = chars[typing.index];
            if c.eq_ignore_ascii_case(&expected) {
                typing.index += 1;
                while typing.index < chars.len() && chars[typing.index] == ' ' {
                    typing.index += 1;
                }
                commands.spawn(
                    SamplePlayer::new(assets.incorrect.clone()).with_volume(Volume::Linear(0.8)),
                );
            } else {
                commands.spawn(
                    SamplePlayer::new(text_assets.presence_glyph.clone())
                        .with_volume(Volume::Linear(0.8)),
                );
            }
        }
    }

    let (typed, rest) = typing.text.split_at(typing.index);
    sections.get_mut(ui[0]).unwrap().0 = typed.to_string();
    sections.get_mut(ui[1]).unwrap().0 = rest.to_string();

    if typing.index >= typing.text.len() {
        commands.entity(*entity).insert(WonMinigame);
    }
}
