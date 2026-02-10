// #[cfg(debug_assertions)]
// use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::{asset::AssetMetaCheck, prelude::*};

mod fractal;
mod minigame;
mod state;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics on web build on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Window {
                    title: "fever".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
        bevy_seedling::SeedlingPlugin::default(),
    ))
    .add_plugins((state::plugin, minigame::plugin, fractal::plugin));

    #[cfg(debug_assertions)]
    app
        //     .add_plugins(FpsOverlayPlugin {
        //     config: FpsOverlayConfig {
        //         ..Default::default()
        //     },
        // })
        .add_systems(Update, close_on_escape);

    app.run();
}

#[cfg(debug_assertions)]
fn close_on_escape(mut writer: MessageWriter<AppExit>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        writer.write(AppExit::Success);
    }
}

// #[derive(Component)]
// struct Key;
//
// fn spawn_key(mut commands: Commands, server: Res<AssetServer>) {
//     let spawn_door = commands.register_system(spawn_door);
//     commands.queue(audio::music_volume("music/rabbit.wav", 0.5));
//     commands.spawn((
//         Key,
//         Collectable("images/key.png"),
//         OnCollect(spawn_door),
//         CPlane,
//         Sprite::from_image(server.load("images/key.png")),
//         c_transform(-0.029999983, -0.7399992).with_scale(Vec3::splat(0.1)),
//         //
//         SamplePlayer::new(server.load("voice/divine-comedy-grain.ogg"))
//             .with_volume(Volume::Linear(0.8))
//             .looping(),
//         sample_effects![
//             FreeverbNode {
//                 room_size: 0.8,
//                 damping: 0.8,
//                 width: 0.5,
//                 ..Default::default()
//             },
//             HrtfNode {
//                 distance_attenuation: DistanceAttenuation {
//                     distance_gain_factor: 0.005,
//                     ..Default::default()
//                 },
//                 ..Default::default()
//             }
//         ],
//     ));
// }
//
// #[derive(Component)]
// struct Door;
//
// fn spawn_door(mut commands: Commands, server: Res<AssetServer>) {
//     let spawn_target_image = commands.register_system(spawn_target_image);
//     commands.queue(audio::music_volume("music/bong.wav", 0.5));
//     commands.spawn((
//         Door,
//         Collectable("images/door.png"),
//         OnCollect(spawn_target_image),
//         CPlane,
//         Sprite::from_image(server.load("images/door.png")),
//         c_transform(0.029999983, 0.7399992).with_scale(Vec3::splat(0.1)),
//         //
//         SamplePlayer::new(server.load("voice/moby-dick-grain.ogg")).looping(),
//         sample_effects![
//             FreeverbNode {
//                 room_size: 0.2,
//                 damping: 1.0,
//                 width: 0.5,
//                 ..Default::default()
//             },
//             HrtfNode {
//                 distance_attenuation: DistanceAttenuation {
//                     distance_gain_factor: 0.005,
//                     ..Default::default()
//                 },
//                 ..Default::default()
//             }
//         ],
//     ));
// }
//
// #[derive(Component)]
// struct CollectedContainer;
//
// fn spawn_collected_container(mut commands: Commands) {
//     commands.spawn((
//         Node {
//             position_type: PositionType::Absolute,
//             right: Val::Px(20.0),
//             top: Val::Px(20.0),
//             flex_direction: FlexDirection::Column,
//             row_gap: Val::Px(10.0),
//             ..default()
//         },
//         CollectedContainer,
//     ));
// }
//
// #[derive(Component)]
// struct Collectable(&'static str);
//
// #[derive(Component)]
// struct Collected;
//
// #[derive(Component)]
// struct OnCollect(SystemId);
//
// fn collect_things(
//     mut commands: Commands,
//     things: Query<(Entity, &Transform, &Collectable, Option<&OnCollect>)>,
//     camera: Single<(&Transform, &Stationary), With<Camera>>,
//     server: Res<AssetServer>,
//     container: Single<Entity, With<CollectedContainer>>,
// ) {
//     let (camera_transform, stationary) = camera.into_inner();
//     if stationary.0.elapsed_secs() < 1.0 {
//         return;
//     }
//
//     for (entity, transform, collectable, on) in things.iter() {
//         let dist = camera_transform
//             .translation
//             .distance_squared(transform.translation);
//         if dist < 500.0 {
//             commands.queue(audio::sfx_volume("sfx/pickup.ogg", 0.75));
//             collect_item(&mut commands, &server, *container, collectable.0);
//             commands.entity(entity).despawn();
//             if let Some(on) = on {
//                 commands.run_system(on.0);
//             }
//         }
//     }
//
//     fn collect_item(commands: &mut Commands, server: &AssetServer, container: Entity, path: &str) {
//         let image = server.load(path.to_string());
//         commands.entity(container).with_children(|parent| {
//             parent.spawn((
//                 Collected,
//                 ImageNode { image, ..default() },
//                 Node {
//                     width: Val::Px(64.0),
//                     height: Val::Px(64.0),
//                     ..default()
//                 },
//             ));
//         });
//     }
// }
