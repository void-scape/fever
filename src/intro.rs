use crate::animation::*;
use crate::audio::LinearVolume;
use crate::audio::Lpf;
use crate::audio::PlaybackSpeed;
use crate::fractal::*;
use crate::minigame::ImageColor;
use crate::minigame::MinigameAssets;
use crate::minigame::UiTranslationPx;
use crate::state::GameState;
use crate::text::*;
use crate::{animations, parallel};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_asset_loader::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;
use fever_macros::Lerp;

pub fn plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<IntroAssets>())
        .add_sub_state::<Intro>()
        .add_systems(OnEnter(Intro::Fade), fade)
        .add_systems(OnEnter(Intro::MoveControls), move_controls)
        .add_systems(OnEnter(Intro::Move), move_state)
        .add_systems(OnEnter(Intro::MouseControls), mouse_controls)
        .add_systems(OnEnter(Intro::Mouse), mouse)
        .add_systems(
            Update,
            update_mouse
                .before(AnimationSystems::Interpolate)
                .run_if(in_state(Intro::Mouse)),
        )
        .add_systems(OnEnter(Intro::FlavorText), flavor_text);
}

#[derive(AssetCollection, Resource)]
struct IntroAssets {
    #[asset(path = "music/rain.ogg")]
    rain: Handle<AudioSample>,
    #[asset(path = "images/fractals/odd-julia.png")]
    odd_julia: Handle<Image>,
    #[asset(path = "images/fractals/pl-julia.png")]
    pl_julia: Handle<Image>,
    #[asset(path = "images/fractals/last-breath.png")]
    last_breath: Handle<Image>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SubStates, Component)]
#[source(GameState = GameState::Intro)]
enum Intro {
    #[default]
    Fade,
    MouseControls,
    Mouse,
    MoveControls,
    Move,
    FlavorText,
}

#[derive(Component)]
struct Music;

fn fade(mut commands: Commands, fractal: Single<Entity, With<Fractal>>, assets: Res<IntroAssets>) {
    let dur = 2.5;
    commands.spawn((
        Music,
        DespawnOnExit(GameState::Intro),
        SamplePlayer::new(assets.rain.clone()).looping(),
        LinearVolume(0.0),
        PlaybackSpeed::default(),
        Lpf::default(),
        sample_effects![
            VolumeNode::default(),
            LowPassNode {
                frequency: 20_000.0
            },
        ],
        AnimationTarget::entity(),
        animations![
            (
                Duration(8.0),
                Keyframe(LinearVolume(1.0)),
                Easing::SineInOut
            ),
            (
                Duration(dur),
                Keyframe(Lpf(10000.0)),
                Keyframe(PlaybackSpeed(0.5)),
                Easing::SineInOut
            ),
            (
                Duration(dur),
                Keyframe(PlaybackSpeed(0.6)),
                Easing::SineInOut
            ),
            (
                Duration(dur),
                Keyframe(PlaybackSpeed(0.2)),
                Easing::SineInOut
            ),
        ],
    ));

    commands.entity(*fractal).insert((
        CPlane(Vec2::new(-0.66333276, 0.42333305)),
        Iterations(0.0),
        Opacity(0.0),
        Zoom(1.5),
    ));
    commands.spawn((
        AnimationTarget(*fractal),
        DespawnFinished,
        animations![
            (Duration(8.0), Keyframe(Opacity(1.0)), Easing::SineInOut),
            (Duration(dur), Keyframe(Iterations(20.0)), Easing::SineInOut),
            (
                Duration(dur),
                Delta(CPlane(Vec2::new(0.1, 0.0))),
                Easing::SineInOut
            ),
            (
                Duration(dur),
                Delta(CPlane(Vec2::new(-0.1, -0.25))),
                Easing::SineInOut
            ),
            set_state(Intro::MouseControls),
        ],
    ));
}

fn mouse_controls(mut commands: Commands, mg_assets: Res<MinigameAssets>) {
    let entity = commands.spawn_empty().id();
    let blocking = blocking_system(
        move |mut commands: Commands, input: MessageReader<MouseMotion>| {
            if !input.is_empty() {
                commands.set_state(Intro::Mouse);
                true
            } else {
                false
            }
        },
    );
    commands
        .entity(entity)
        .insert(controls_bundle(entity, mg_assets.mouse.clone(), blocking));
}

fn mouse(mut commands: Commands, music: Single<Entity, With<Music>>) {
    commands.spawn((
        PathOpacity(0.0),
        AnimationTarget::entity(),
        DespawnFinished,
        animations![
            (Duration(2.0), Keyframe(PathOpacity(1.0)), Easing::SineInOut),
            Duration(3.0),
            parallel![
                (Duration(2.0), Keyframe(PathOpacity(0.0)), Easing::SineInOut),
                (
                    AnimationTarget(*music),
                    Duration(2.0),
                    Keyframe(Lpf(20000.0)),
                    Easing::SineInOut
                ),
            ],
            set_state(Intro::MoveControls),
        ],
    ));
}

#[derive(Clone, Copy, Component, Lerp)]
struct PathOpacity(f32);

fn update_mouse(
    mut gizmos: Gizmos,
    mut lpf: Single<&mut Lpf, With<Music>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    opacity: Single<&PathOpacity>,
    zoom: Single<&Zoom, With<Fractal>>,
) {
    let (camera, camera_transform) = camera.into_inner();
    if let Some(w) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        let c = Vec2::new(w_to_c(w.x, zoom.0), w_to_c(w.y, zoom.0));
        let mut path = vec![Vec2::ZERO];
        let mut z = Vec2::ZERO;
        for _ in 0..100 {
            if z.length_squared() > 4.0 * 4.0 {
                break;
            }
            z = cmul(z, z) + c;
            path.push(z);
        }

        if path.len() <= 1 {
            return;
        }

        for points in path.windows(2) {
            gizmos.line_2d(
                Vec2::new(c_to_w(points[0].x, zoom.0), c_to_w(points[0].y, zoom.0)),
                Vec2::new(c_to_w(points[1].x, zoom.0), c_to_w(points[1].y, zoom.0)),
                Color::WHITE.with_alpha(opacity.0),
            );
        }

        // means the lpf is not getting animated
        if opacity.0 > 0.25 {
            let max = path
                .iter()
                .map(|p| p.length_squared())
                .reduce(f32::max)
                .unwrap()
                .clamp(0.0, 500.0);
            **lpf = Lpf::distance(500.0 - max, 500.0);
        }
    }
}

fn move_controls(mut commands: Commands, mg_assets: Res<MinigameAssets>) {
    let entity = commands.spawn_empty().id();
    let blocking = blocking_system(
        move |mut commands: Commands, input: Res<ButtonInput<KeyCode>>| {
            let codes = [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD];
            if input.get_pressed().any(|k| codes.contains(k)) {
                commands.set_state(Intro::Move);
                true
            } else {
                false
            }
        },
    );
    commands
        .entity(entity)
        .insert(controls_bundle(entity, mg_assets.wasd.clone(), blocking));
}

fn move_state(
    mut commands: Commands,
    sens: Single<Entity, With<MovementSensitivity>>,
    music: Single<Entity, With<Music>>,
    fractal: Single<Entity, With<Fractal>>,
) {
    commands.spawn((
        DespawnFinished,
        animations![
            Duration(5.0),
            parallel![
                (
                    AnimationTarget(*sens),
                    Duration(3.0),
                    Keyframe(MovementSensitivity(0.0)),
                    Easing::SineInOut,
                ),
                (
                    AnimationTarget(*music),
                    Duration(3.0),
                    Keyframe(PlaybackSpeed(0.1)),
                    Easing::SineInOut,
                ),
            ],
            Duration(1.0),
            parallel![
                (
                    AnimationTarget(*music),
                    Duration(5.0),
                    Keyframe(PlaybackSpeed(1.0)),
                    Easing::SineInOut
                ),
                (
                    AnimationTarget(*fractal),
                    Duration(5.0),
                    Keyframe(Zoom(100.0)),
                    Keyframe(Opacity(0.0)),
                    Easing::SineInOut
                )
            ],
            set_state(Intro::FlavorText),
        ],
    ));
}

fn flavor_text(mut commands: Commands) {
    commands.spawn((
        DespawnFinished,
        animations![
            system(
                |fractal: Single<
                    (
                        &mut Zoom,
                        &mut Opacity,
                        &mut CPlane,
                        &mut Iterations,
                        &mut FractalTexture
                    ),
                    With<Fractal>,
                >,
                 assets: Res<IntroAssets>| {
                    let (mut zoom, mut opacity, mut cplane, mut iterations, mut texture) =
                        fractal.into_inner();
                    zoom.0 = 1.5;
                    opacity.0 = 1.0;
                    cplane.0 = Vec2::ZERO;
                    iterations.0 = 0.0;
                    texture.0 = assets.odd_julia.clone();
                }
            ),
            await_input(pretty!(
                "|1|How long have you been there,|0.5|<0.85> [watching me](glitch, red)?"
            )),
            system(
                |mut texture: Single<&mut FractalTexture, With<Fractal>>,
                 assets: Res<IntroAssets>| {
                    texture.0 = assets.last_breath.clone();
                }
            ),
            await_finish(pretty!("Do you|0.1| hate [me](glitch, red)?|0.25|")),
            system(
                |mut texture: Single<&mut FractalTexture, With<Fractal>>,
                 assets: Res<IntroAssets>| {
                    texture.0 = assets.pl_julia.clone();
                }
            ),
            await_input(pretty!(
                "|1|<0.9>I move my hand over you|0.25|<1.15> but it<1> does not block your \
                <0.8>[bleeding glow](glitch, red).|1| You must be|0.25|<1.25> imaginary."
            )),
            set_state(GameState::Playing),
        ],
    ));
}

fn controls_bundle(
    entity: Entity,
    image: Handle<Image>,
    blocking: crate::animation::System,
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
        UiTranslationPx(Vec2::new(0.0, -50.0)),
        //
        AnimationTarget::entity(),
        DespawnFinished,
        animations![
            (
                Duration(2.0),
                Keyframe(ImageColor(Color::srgba(1.0, 1.0, 1.0, 1.0))),
                Keyframe(UiTranslationPx(Vec2::ZERO)),
                Easing::SineInOut,
            ),
            system(move |mut commands: Commands| {
                commands.entity(entity).with_child((
                    Loop,
                    AnimationTarget(entity),
                    animations![
                        (
                            Duration(0.8),
                            Keyframe(UiTranslationPx(Vec2::new(0.0, -10.0))),
                            Easing::SineInOut
                        ),
                        (
                            Duration(0.8),
                            Keyframe(UiTranslationPx(Vec2::new(0.0, 0.0))),
                            Easing::SineInOut
                        ),
                    ],
                ));
            }),
            blocking,
            (
                Duration(1.0),
                Keyframe(ImageColor(Color::srgba(1.0, 1.0, 1.0, 0.0))),
                Easing::SineInOut,
            ),
        ],
    )
}
