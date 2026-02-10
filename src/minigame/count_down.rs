use crate::{minigame::Minigame, state::GameState};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_seedling::{
    prelude::Volume,
    sample::{AudioSample, SamplePlayer},
};

pub fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(GameState::Loading).load_collection::<CountDownAssets>(),
    )
    .add_systems(OnEnter(Minigame::Countdown), start_count_down)
    .add_systems(Update, count_down.run_if(in_state(Minigame::Countdown)));
}

#[derive(AssetCollection, Resource)]
struct CountDownAssets {
    #[asset(path = "sfx/boom.ogg")]
    boom: Handle<AudioSample>,
    #[asset(path = "sfx/start.ogg")]
    start: Handle<AudioSample>,
    #[asset(path = "images/one.png")]
    one: Handle<Image>,
    #[asset(path = "images/two.png")]
    two: Handle<Image>,
    #[asset(path = "images/three.png")]
    three: Handle<Image>,
}

#[derive(Component)]
struct CountDown {
    timer: Timer,
    index: usize,
}

fn start_count_down(mut commands: Commands, assets: Res<CountDownAssets>) {
    commands.spawn(SamplePlayer::new(assets.boom.clone()).with_volume(Volume::Linear(0.8)));
    commands.spawn((
        DespawnOnExit(Minigame::Countdown),
        CountDown {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            index: 0,
        },
        ImageNode::new(assets.three.clone()),
        ZIndex(10_000),
        Node {
            height: percent(100.0),
            ..Default::default()
        },
    ));
}

fn count_down(
    mut commands: Commands,
    time: Res<Time>,
    count_down: Single<(Entity, &mut CountDown, &mut ImageNode)>,
    assets: Res<CountDownAssets>,
) {
    let (entity, mut cd, mut sprite) = count_down.into_inner();
    cd.timer.tick(time.delta());
    if cd.timer.just_finished() {
        cd.index += 1;
        match cd.index {
            1 => {
                commands
                    .spawn(SamplePlayer::new(assets.boom.clone()).with_volume(Volume::Linear(0.8)));
                sprite.image = assets.two.clone();
            }
            2 => {
                commands
                    .spawn(SamplePlayer::new(assets.boom.clone()).with_volume(Volume::Linear(0.8)));
                sprite.image = assets.one.clone();
            }
            _ => {
                commands.spawn(
                    SamplePlayer::new(assets.start.clone()).with_volume(Volume::Linear(0.8)),
                );
                commands.entity(entity).despawn();
                commands.set_state(Minigame::Choose);
            }
        }
    }
}
