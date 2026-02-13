use crate::animation::*;
use crate::animations;
use crate::camera::lock_camera;
use crate::text::{await_finish, await_input};
use crate::{
    fractal::{Fractal, Opacity},
    minigame::{
        AvailableAfter, Description, Minigame, MinigameRoot, NotRandom, OnVariationEnable,
        Variation, VariationSet,
    },
    state::GameState,
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<DreamAssets>())
        .add_systems(OnEnter(GameState::Playing), init_targets)
        .add_systems(OnEnter(Minigame::Dream), lock_camera);
}

#[derive(AssetCollection, Resource)]
struct DreamAssets {
    #[asset(path = "music/rain.ogg")]
    rain: Handle<AudioSample>,
    #[asset(path = "music/deep.ogg")]
    deep: Handle<AudioSample>,
    #[asset(path = "music/birds.ogg")]
    birds: Handle<AudioSample>,
    #[asset(path = "music/hell.ogg")]
    hell: Handle<AudioSample>,
}

#[derive(Component)]
struct Dream;

fn init_targets(mut commands: Commands, assets: Res<DreamAssets>) {
    let variations = children![(
        VariationSet,
        NotRandom,
        children![
            variation(
                &mut commands,
                (
                    SamplePlayer::new(assets.deep.clone())
                        .with_volume(Volume::Linear(1.0))
                        .looping(),
                    DespawnFinished,
                    animations![Duration(5.0), set_state(Minigame::Success)]
                )
            ),
            variation(
                &mut commands,
                (
                    SamplePlayer::new(assets.rain.clone())
                        .with_volume(Volume::Linear(1.0))
                        .looping(),
                    DespawnFinished,
                    animations![
                        await_input(pretty!(
                            "|2|You|0.25| should not|0.5| be [here](shake, red)<0.5>..."
                        )),
                        set_state(Minigame::Success)
                    ]
                )
            ),
            variation(
                &mut commands,
                (
                    SamplePlayer::new(assets.birds.clone())
                        .with_volume(Volume::Linear(1.0))
                        .looping(),
                    DespawnFinished,
                    animations![
                        await_input(pretty!("|2|There is no end.")),
                        await_input(pretty!(
                            "A [dream](red) with no beginning has no end<0.5>..."
                        )),
                        set_state(Minigame::Success)
                    ]
                )
            ),
            variation(
                &mut commands,
                (
                    SamplePlayer::new(assets.hell.clone())
                        .with_volume(Volume::Linear(1.0))
                        .looping(),
                    DespawnFinished,
                    animations![
                        await_input(pretty!("What do you seek in this [dream](red)?")),
                        await_input(pretty!("This [dream](red) will only take from you.")),
                        await_finish(pretty!("|1.0|How did you get [here](red)?|0.25|")),
                        set_state(Minigame::Success)
                    ]
                )
            ),
        ],
    )];

    commands.spawn((
        MinigameRoot,
        DespawnOnExit(GameState::Playing),
        AvailableAfter(6),
        Minigame::Dream,
        Dream,
        Description::Ear,
        variations,
    ));

    fn variation(commands: &mut Commands, bundle: impl Bundle) -> impl Bundle {
        let mut bundle = Some(bundle);
        let on_start = OnVariationEnable(commands.register_system(
            move |_: In<Entity>,
                  mut commands: Commands,
                  mut opacity: Single<&mut Opacity, With<Fractal>>| {
                opacity.0 = 0.0;
                if let Some(bundle) = bundle.take() {
                    commands.spawn(bundle);
                }
            },
        ));
        (Variation, on_start)
    }
}
