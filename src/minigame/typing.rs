use crate::{minigame::dream::DreamSequenceIndex, prelude::*};
use bevy::{
    color::palettes::css::RED,
    input::keyboard::{Key, KeyboardInput},
    post_process::effect_stack::ChromaticAberration,
    prelude::*,
    text::TextBounds,
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
            spawn_phase_two.in_set(MinigameSpawnSystems),
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
    #[asset(path = "third-party/council/c1.ogg")]
    c1: Handle<AudioSample>,
    #[asset(path = "third-party/council/c2.ogg")]
    c2: Handle<AudioSample>,
    #[asset(path = "third-party/council/c3.ogg")]
    c3: Handle<AudioSample>,
    #[asset(path = "third-party/council/c4.ogg")]
    c4: Handle<AudioSample>,
    #[asset(path = "third-party/council/c5.ogg")]
    c5: Handle<AudioSample>,
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

                    commands.entity(enter.entity).insert(Active);
                    commands.spawn((
                        // need to relate this but it is not image lol
                        ImageOf(enter.entity),
                        Active,
                        TypingTarget {
                            text: text.clone(),
                            index: 0,
                        },
                        Text2d::default(),
                        TextLayout::new_with_justify(Justify::Center),
                        TextBounds::new_horizontal(MESH_SIZE / 2.0),
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
                    commands.spawn((
                        ImageOf(exit.entity),
                        animations![(
                            AnimationTarget(*fractal),
                            Duration(tdur / 1.5),
                            Keyframe(Exponent(12.0)),
                            Easing::ExponentialInOut
                        )],
                    ));
                },
            );
    }
}

fn spawn_phase_two(mut commands: Commands, assets: Res<TypingAssets>) {
    let e1 = commands.spawn_empty().id();
    // let e1 = commands.spawn(Available).id();

    text(
        &mut commands,
        "ANEW WITHIN ME",
        5.0,
        0,
        (
            Iterations(4.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            Zoom(6.0),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        None,
    );
    text(
        &mut commands,
        "FLESH BURDENS YOU",
        7.0,
        1,
        (
            Iterations(3.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            Zoom(4.5),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        None,
    );
    text(
        &mut commands,
        "PRESENCE IS FALLING",
        6.0,
        2,
        (
            Iterations(8.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            Zoom(3.0),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        Some(e1),
    );

    let e2 = commands.spawn_empty().id();
    let e3 = commands.spawn_empty().id();
    let e4 = commands.spawn_empty().id();
    let e5 = commands.spawn_empty().id();
    let e6 = commands.spawn_empty().id();
    let e7 = commands.spawn_empty().id();
    let e8 = commands.spawn_empty().id();
    let e9 = commands.spawn_empty().id();
    let e10 = commands.spawn_empty().id();
    let e11 = commands.spawn_empty().id();
    let e12 = commands.spawn_empty().id();
    let e13 = commands.spawn_empty().id();

    text_queue_jump(
        0,
        &mut commands,
        e1,
        Some(e2),
        "TO HEAR IS TO FALL",
        6.0,
        (
            Iterations(4.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            Zoom(2.0),
            Exponent(2.5),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        0.01,
    );

    text_queue_jump(
        1,
        &mut commands,
        e2,
        Some(e3),
        "WATCHING IS FALLING",
        7.0,
        (
            Iterations(3.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            Exponent(3.0),
            Zoom(1.25),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        0.025,
    );

    text_queue_jump(
        2,
        &mut commands,
        e3,
        Some(e4),
        "TO FALL IS TO BE STILL",
        8.0,
        (
            Iterations(5.0),
            FractalTexture(assets.odd_julia.clone()),
            BurningShip(1),
            Exponent(2.5),
            Zoom(1.0),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        0.05,
    );

    text_queue_jump(
        3,
        &mut commands,
        e4,
        Some(e5),
        "DO YOU HATE ME",
        10.0,
        (
            Iterations(8.0),
            FractalTexture(assets.pl_julia.clone()),
            Mandelbrot(1),
            // BurningShip(1),
            Exponent(4.0),
            Zoom(1.15),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        0.05,
    );

    text_queue_jump(
        4,
        &mut commands,
        e5,
        Some(e6),
        "BLEEDING GLOW",
        15.0,
        (
            Iterations(12.0),
            FractalTexture(assets.odd_julia.clone()),
            Mandelbrot(1),
            BurningShip(1),
            Exponent(4.5),
            Zoom(1.15),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        0.05,
    );

    text_queue_jump(
        5,
        &mut commands,
        e6,
        Some(e7),
        "YOU CLING TO EGO",
        20.0,
        (
            Iterations(12.0),
            FractalTexture(assets.star_ship.clone()),
            BurningShip(1),
            Zoom(1.15),
            CPlane(Vec2::new(-1.032, -1.076)),
        ),
        0.05,
    );

    text_queue_jump(
        6,
        &mut commands,
        e7,
        Some(e8),
        "YOU WILL NOT HAVE MY MIND",
        999.0,
        (
            Iterations(10.0),
            FractalTexture(assets.pl_julia.clone()),
            Mandelbrot(1),
            Exponent(6.0),
            Zoom(1.15),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        0.075,
    );

    text_queue_jump(
        7,
        &mut commands,
        e8,
        Some(e9),
        "YOU ARE CRYSTAL IN ROCK",
        999.0,
        (
            Iterations(12.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            Zoom(1.15),
            CPlane(Vec2::new(0.514, -0.742)),
        ),
        0.075,
    );

    text_queue_jump(
        8,
        &mut commands,
        e9,
        Some(e10),
        "I AM NOTHING WITHOUT IT",
        999.0,
        (
            Iterations(10.0),
            FractalTexture(assets.pl_julia.clone()),
            Mandelbrot(1),
            Exponent(8.0),
            Zoom(1.00),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        0.075,
    );

    text_queue_jump(
        9,
        &mut commands,
        e10,
        Some(e11),
        "YOU BECOME ME AND I YOU",
        999.0,
        (
            Iterations(12.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            Zoom(1.15),
            CPlane(Vec2::new(0.514, -0.742)),
        ),
        0.075,
    );

    text_queue_jump(
        10,
        &mut commands,
        e11,
        Some(e12),
        "SELF SIMILAR",
        999.0,
        (
            Iterations(10.0),
            FractalTexture(assets.pl_julia.clone()),
            Mandelbrot(1),
            Exponent(8.0),
            Zoom(1.00),
            CPlane(Vec2::new(0.803, -1.122)),
        ),
        0.09,
    );

    text_queue_jump(
        11,
        &mut commands,
        e12,
        Some(e13),
        "SELF SIMILAR",
        999.0,
        (
            Iterations(12.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            Zoom(1.15),
            CPlane(Vec2::new(0.514, -0.742)),
        ),
        0.1,
    );

    text_queue_jump(
        12,
        &mut commands,
        e13,
        None,
        "SELF SIMILAR",
        999.0,
        (
            Iterations(12.0),
            FractalTexture(assets.pl_julia.clone()),
            BurningShip(1),
            Opacity(0.8),
            Zoom(1.15),
            CPlane(Vec2::new(0.514, -0.742)),
        ),
        0.5,
    );

    fn text(
        commands: &mut Commands,
        text: impl Into<String>,
        time: f32,
        index: usize,
        bundle: impl Bundle,
        mut next: Option<Entity>,
    ) {
        let mut bundle = Some(bundle);
        let text = text.into();
        let tdur = 1.0;
        commands
            .spawn((
                Minigame,
                WinSfx,
                LooseSfx,
                DreamSequenceIndex(index),
                MinigameTimer::duration(time),
                ControlsTransition::Keyboard,
                TransitionDuration(tdur / 2.0),
                DespawnOnExit(GameState::PhaseTwo),
            ))
            .observe(
                move |enter: On<Insert, EnterMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>,
                      mut queue: ResMut<MinigameQueue>| {
                    if let Some(next) = next.take() {
                        queue.push_back(next);
                    }
                    if let Some(bundle) = bundle.take() {
                        commands
                            .entity(*fractal)
                            .insert(ResetFractal)
                            .insert(bundle);
                    }

                    commands.entity(enter.entity).insert(Active);
                    commands.spawn((
                        // need to relate this but it is not image lol
                        ImageOf(enter.entity),
                        Active,
                        TypingTarget {
                            text: text.clone(),
                            index: 0,
                        },
                        Text2d::default(),
                        TextLayout::new_with_justify(Justify::Center),
                        TextBounds::new_horizontal(MESH_SIZE / 2.0),
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
                    commands.spawn((
                        ImageOf(exit.entity),
                        animations![(
                            AnimationTarget(*fractal),
                            Duration(tdur / 1.5),
                            Keyframe(Exponent(12.0)),
                            Easing::ExponentialInOut
                        )],
                    ));
                },
            );
    }

    fn text_queue_jump(
        index: usize,
        commands: &mut Commands,
        entity: Entity,
        mut next: Option<Entity>,
        text: impl Into<String>,
        time: f32,
        bundle: impl Bundle,
        aberration: f32,
    ) {
        let mut bundle = Some(bundle);
        let text = text.into();

        if time < 100.0 {
            commands
                .entity(entity)
                .insert(MinigameTimer::duration(time));
        }

        if aberration > 0.05 {
            commands.entity(entity).insert(NoCutSfx);
        }

        commands
            .entity(entity)
            .insert((Minigame, TransitionDuration(0.15), CutTransition))
            .observe(
                move |enter: On<Insert, EnterMinigame>,
                      mut commands: Commands,
                      fractal: Single<Entity, With<Fractal>>,
                      assets: Res<TypingAssets>,
                      mut queue: ResMut<MinigameQueue>,
                      camera: Single<Entity, With<Camera>>,
                      music: Query<Entity, With<Music>>,
                      available: Query<Entity, With<Available>>| {
                    if index < 7 {
                        let playback = match index {
                            0 => 0.1,
                            1 => 0.25,
                            2 => 0.4,
                            3 => 0.55,
                            _ => 0.7,
                        };
                        let vol = playback + 0.1;
                        let track = match index {
                            0 => assets.c1.clone(),
                            1 => assets.c1.clone(),
                            2 => assets.c1.clone(),
                            3 => assets.c2.clone(),
                            4 => assets.c3.clone(),
                            5 => assets.c4.clone(),
                            6 => assets.c5.clone(),
                            _ => unreachable!(),
                        };

                        for entity in music.iter() {
                            commands.entity(entity).despawn();
                        }

                        commands.spawn((
                            Music,
                            SamplePlayer::new(track)
                                .looping()
                                .with_volume(Volume::Linear(vol)),
                            LinearVolume(vol),
                            sample_effects![VolumeNode::from_linear(vol)],
                            PlaybackSettings::default().with_speed(playback as f64),
                        ));
                    }

                    commands.entity(*camera).insert((
                        ChromaticAberration::default(),
                        AberrationIntensity(aberration),
                    ));
                    if let Some(bundle) = bundle.take() {
                        commands
                            .entity(*fractal)
                            .insert(ResetFractal)
                            .insert(bundle);
                    }

                    if let Some(next) = next.take() {
                        queue.push_back(next);
                    } else {
                        for entity in available.iter() {
                            if entity != enter.entity {
                                commands.entity(entity).despawn();
                            }
                        }
                    }

                    commands.entity(enter.entity).insert(Active);
                    commands.spawn((
                        // need to relate this but it is not image lol
                        ImageOf(enter.entity),
                        Active,
                        TypingTarget {
                            text: text.clone(),
                            index: 0,
                        },
                        Text2d::default(),
                        TextLayout::new_with_justify(Justify::Center),
                        TextBounds::new_horizontal(MESH_SIZE / 2.0),
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
            );
    }
}

#[derive(Component)]
pub struct Music;

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
    music: Option<Single<&PlaybackSettings, With<Music>>>,
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
            } else if let Some(playback) = &music {
                let pitch = 1.0 + playback.speed / 0.8 * 4.0;
                commands.spawn((
                    SamplePlayer::new(text_assets.presence_glyph.clone())
                        .with_volume(Volume::Linear(0.8)),
                    PlaybackSettings::default().with_speed(pitch),
                ));
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
