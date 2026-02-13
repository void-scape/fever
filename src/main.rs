// #[cfg(debug_assertions)]
// use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_rand::prelude::WyRand;

#[allow(unused)]
mod animation;
mod audio;
mod camera;
mod fractal;
mod intro;
mod minigame;
mod state;
mod text;

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
        bevy_rand::plugin::EntropyPlugin::<WyRand>::with_seed(69u64.to_le_bytes()),
        bevy_pretty_text::prelude::PrettyTextPlugin,
    ))
    .add_plugins((
        state::plugin,
        minigame::plugin,
        fractal::plugin,
        audio::plugin,
        intro::plugin,
        animation::plugin,
        text::plugin,
        camera::plugin,
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
