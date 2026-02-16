use crate::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;

pub fn outro_plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<OutroAssets>())
        .add_systems(OnEnter(GameState::Outro), animation);
}

#[derive(AssetCollection, Resource)]
struct OutroAssets {
    #[asset(path = "third-party/wind.ogg")]
    wind: Handle<AudioSample>,
}

fn animation(
    mut commands: Commands,
    assets: Res<OutroAssets>,
    text: Query<(Entity, &TextColor)>,
    music: Single<Entity, With<typing::Music>>,
    fractal: Single<Entity, With<Fractal>>,
    camera: Single<Entity, With<Camera>>,
) {
    for (entity, color) in text.iter() {
        commands
            .entity(entity)
            .insert(TextColorTween(color.0))
            .with_child(animations![(
                AnimationTarget(entity),
                Duration(5.0),
                Keyframe(TextColorTween(color.0.with_alpha(0.0))),
                Easing::SineInOut
            )]);
    }

    let wind = commands
        .spawn(
            SamplerBuilder::new(SamplePlayer::new(assets.wind.clone()).looping())
                .volume(0.0)
                .build(),
        )
        .id();

    commands.spawn(animations![
        Duration(2.5),
        parallel![
            (
                AnimationTarget(*music),
                Duration(20.0),
                Keyframe(LinearVolume(0.0)),
                Easing::SineInOut,
            ),
            (
                AnimationTarget(wind),
                Duration(20.0),
                Keyframe(LinearVolume(0.5)),
                Easing::SineInOut,
            ),
            (
                AnimationTarget(*camera),
                Duration(15.0),
                Keyframe(AberrationIntensity(0.0)),
                Easing::SineInOut,
            ),
            animations![
                (
                    AnimationTarget(*fractal),
                    Duration(25.0),
                    Keyframe(Iterations(0.0)),
                    Easing::SineInOut,
                ),
                (
                    AnimationTarget(*fractal),
                    Duration(10.0),
                    Keyframe(Opacity(0.0)),
                    Easing::SineInOut,
                )
            ],
        ],
        system(|mut writer: MessageWriter<AppExit>| {
            writer.write(AppExit::Success);
        })
    ]);
}
