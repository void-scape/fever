use crate::{
    audio::Lpf,
    fractal::{FractalUniform, Params, c_to_w, ctransform, lock_camera, w_to_c},
    minigame::{Description, Minigame, MinigameRoot, StartTimer, Variation, VariationSet},
    state::GameState,
};
use bevy::{
    color::palettes::tailwind::GREEN_800, ecs::entity_disabling::Disabled, prelude::*,
    window::PrimaryWindow,
};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<PathAssets>())
        .add_systems(OnEnter(GameState::Playing), init_targets)
        .add_systems(OnEnter(Minigame::Path), lock_camera)
        .add_systems(
            Update,
            (intersect_targets, check_success)
                .chain()
                .run_if(in_state(Minigame::Path)),
        );
}

#[derive(AssetCollection, Resource)]
struct PathAssets {
    // TODO: different images
    #[asset(path = "images/fractals/god.png")]
    god: Handle<Image>,
    //
    // TODO: different music
    #[asset(path = "music/bong.wav")]
    bong: Handle<AudioSample>,
    #[asset(path = "music/rabbit.wav")]
    rabbit: Handle<AudioSample>,
    //
    #[asset(path = "sfx/hovered.ogg")]
    hovered: Handle<AudioSample>,
    #[asset(path = "sfx/unhovered.ogg")]
    unhovered: Handle<AudioSample>,
}

#[derive(Component)]
#[require(Transform)]
struct TargetRoot;

#[derive(Component)]
struct Target(f32);

fn init_targets(
    mut commands: Commands,
    mut config_store: ResMut<GizmoConfigStore>,
    assets: Res<PathAssets>,
) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = 5.0;

    commands.spawn((
        MinigameRoot,
        Minigame::Path,
        Description("INTERSECT TARGETS\n(MOUSE)"),
        children![(
            VariationSet,
            children![
                target(
                    10.0,
                    assets.god.clone(),
                    assets.rabbit.clone(),
                    children![
                        (Disabled, Target(10.0), ctransform(-0.5627136, 0.0018424888)),
                        (Disabled, Target(10.0), ctransform(-0.79316336, -0.35562518)),
                        (Disabled, Target(10.0), ctransform(-0.18567663, -0.57410425)),
                    ],
                ),
                target(
                    10.0,
                    assets.god.clone(),
                    assets.bong.clone(),
                    children![
                        (Disabled, Target(10.0), ctransform(-0.8701706, 0.20652005)),
                        (
                            Disabled,
                            Target(10.0),
                            ctransform(-0.73051834, -0.085132554)
                        ),
                        (Disabled, Target(10.0), ctransform(-0.5151634, -0.14848703)),
                        (Disabled, Target(10.0), ctransform(-0.33082193, -0.12422559)),
                        (Disabled, Target(10.0), ctransform(-0.14261243, 0.461071)),
                        (Disabled, Target(10.0), ctransform(-0.4313813, 0.48950952)),
                        (Disabled, Target(10.0), ctransform(-0.57834625, 0.41619867)),
                    ],
                ),
            ]
        )],
    ));

    fn target(
        time: f32,
        texture: Handle<Image>,
        music: Handle<AudioSample>,
        targets: impl Bundle,
    ) -> impl Bundle {
        (
            Variation,
            StartTimer(time),
            TargetRoot,
            targets,
            FractalUniform {
                texture,
                params: Params {
                    cx: -0.5,
                    mandelbrot: 1,
                    exponent: 2.0,
                    ..Default::default()
                },
            },
            SamplePlayer::new(music)
                .with_volume(Volume::Linear(0.8))
                .looping(),
            sample_effects![LowPassNode {
                frequency: 20_000.0
            }],
            Lpf(20_000.0),
        )
    }
}

#[derive(Component)]
struct Hovered;

fn intersect_targets(
    mut commands: Commands,
    mut gizmos: Gizmos,
    targets: Query<(Entity, &Target, &Transform, Has<Hovered>)>,
    mut lpf: Single<&mut Lpf, With<TargetRoot>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    assets: Res<PathAssets>,
    //
    // input: Res<ButtonInput<KeyCode>>,
    // mut position: Local<Vec2>,
) {
    for (_, target, transform, is_hovered) in targets.iter() {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.xy()),
            target.0,
            if is_hovered {
                GREEN_800.into()
            } else {
                Color::WHITE
            },
        );
    }

    let (camera, camera_transform) = camera.into_inner();
    if let Some(w) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        let c = Vec2::new(w_to_c(w.x), w_to_c(w.y));

        // if input.pressed(KeyCode::ShiftLeft) {
        //     *position = c;
        // }
        // if input.just_pressed(KeyCode::KeyA) {
        //     println!("{c}");
        // }
        // let c = *position;

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
                Vec2::new(c_to_w(points[0].x), c_to_w(points[0].y)),
                Vec2::new(c_to_w(points[1].x), c_to_w(points[1].y)),
                Color::WHITE,
            );
        }

        let mut hov = false;
        let mut unhov = false;
        'outer: for (entity, target, transform, hovered) in targets.iter() {
            for point in path.iter() {
                let dist = Vec2::new(c_to_w(point.x), c_to_w(point.y))
                    .distance_squared(transform.translation.xy());
                if dist < target.0 * target.0 {
                    if hovered {
                        continue 'outer;
                    }

                    hov = true;
                    commands.entity(entity).insert(Hovered);
                    continue 'outer;
                }
            }
            if hovered {
                unhov = true;
                commands.entity(entity).remove::<Hovered>();
            }
        }

        if hov {
            commands
                .spawn(SamplePlayer::new(assets.hovered.clone()).with_volume(Volume::Linear(0.8)));
        }
        if unhov {
            commands.spawn(
                SamplePlayer::new(assets.unhovered.clone()).with_volume(Volume::Linear(0.8)),
            );
        }

        let len = targets.iter().len() as f32;
        **lpf = Lpf::distance(
            len - targets.iter().filter(|(_, _, _, h)| *h).count() as f32,
            len,
        );
    }

    fn cmul(a: Vec2, b: Vec2) -> Vec2 {
        Vec2::new(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x)
    }
}

fn check_success(
    mut commands: Commands,
    targets: Query<(Entity, &Target, &Transform), Without<Hovered>>,
) {
    if targets.is_empty() {
        commands.set_state(Minigame::Success);
    }
}
