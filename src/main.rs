// #[cfg(debug_assertions)]
// use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_rand::prelude::WyRand;

#[allow(unused)]
pub mod prelude {
    pub use super::animation::*;
    pub use super::audio::*;
    pub use super::camera::*;
    pub use super::fractal::*;
    pub use super::minigame::*;
    pub use super::state::*;
    pub use super::text::*;
    pub use super::tween::*;
    pub use super::{ImageOf, Images, animations, parallel};
    pub use bevy_rand::{global::GlobalRng, prelude::WyRand};
}

#[allow(unused)]
pub mod animation;
pub mod audio;
pub mod camera;
pub mod fractal;
pub mod intro;
pub mod minigame;
pub mod outro;
pub mod restart;
pub mod state;
pub mod text;
pub mod tween;

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
        #[cfg(feature = "dev")]
        bevy_rand::plugin::EntropyPlugin::<WyRand>::default(),
        // bevy_rand::plugin::EntropyPlugin::<WyRand>::with_seed(69u64.to_le_bytes()),
        #[cfg(not(feature = "dev"))]
        bevy_rand::plugin::EntropyPlugin::<WyRand>::default(),
        bevy_pretty_text::prelude::PrettyTextPlugin,
    ))
    .add_plugins((
        state::state_plugin,
        minigame::minigame_plugin,
        fractal::fractal_plugin,
        audio::audio_plugin,
        intro::intro_plugin,
        animation::animation_plugin,
        text::text_plugin,
        camera::camera_plugin,
        outro::outro_plugin,
        tween::tween_plugin,
        restart::restart_plugin,
    ))
    .add_systems(Startup, gizmos_line_width);

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

fn gizmos_line_width(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = 5.0;
}

#[derive(Component)]
#[relationship_target(relationship = ImageOf, linked_spawn)]
pub struct Images(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Images)]
pub struct ImageOf(pub Entity);
