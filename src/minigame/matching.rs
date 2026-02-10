use crate::{
    fractal::{FractalUniform, Params, Stationary, c_to_w, ctransform},
    minigame::Minigame,
    state::GameState,
};
use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(GameState::Loading).load_collection::<MatchingAssets>(),
    )
    .add_systems(OnEnter(GameState::Playing), init_targets)
    // TODO: Fix the flashing in here, somehow the params are not updated after the
    // count down screen in despawned.
    .add_systems(OnEnter(Minigame::Matching), assign_target)
    .add_systems(Update, reach_target.run_if(in_state(Minigame::Matching)));
}

#[derive(AssetCollection, Resource)]
struct MatchingAssets {
    #[asset(path = "images/pickover.png")]
    pickover: Handle<Image>,
    //
    #[asset(path = "images/targets/0.png")]
    t0: Handle<Image>,
    #[asset(path = "images/targets/1.png")]
    t1: Handle<Image>,
    //
    #[asset(path = "music/bong.wav")]
    bong: Handle<AudioSample>,
    #[asset(path = "music/rabbit.wav")]
    rabbit: Handle<AudioSample>,
}

#[derive(Component)]
pub struct TargetPosition;

#[derive(Component)]
pub struct TargetImage(Handle<Image>);

fn init_targets(mut commands: Commands, assets: Res<MatchingAssets>) {
    commands.spawn((
        markers(),
        ctransform(0.0, -0.67999965),
        TargetImage(assets.t0.clone()),
        SamplePlayer::new(assets.bong.clone())
            .with_volume(Volume::Linear(0.8))
            .looping(),
        FractalUniform {
            texture: assets.pickover.clone(),
            params: Params::default(),
        },
    ));

    commands.spawn((
        markers(),
        ctransform(-1.2599992, 0.0),
        TargetImage(assets.t1.clone()),
        SamplePlayer::new(assets.rabbit.clone())
            .with_volume(Volume::Linear(0.8))
            .looping(),
        FractalUniform {
            texture: assets.pickover.clone(),
            params: Params::default(),
        },
    ));

    fn markers() -> impl Bundle {
        (Disabled, MusicPool, TargetPosition)
    }
}

pub fn assign_target(
    mut commands: Commands,
    targets: Query<(Entity, &FractalUniform, &TargetImage), (With<Disabled>, With<TargetPosition>)>,
    mut assets: ResMut<Assets<FractalUniform>>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    let (entity, uniform, image) = targets.iter().next().unwrap();
    commands
        .entity(entity)
        .remove::<Disabled>()
        .insert(image_bundle(image.0.clone()));
    camera.translation.x = c_to_w(uniform.params.cx);
    camera.translation.y = c_to_w(uniform.params.cy);
    for (_, fractal) in assets.iter_mut() {
        fractal.texture = uniform.texture.clone();
        fractal.params = uniform.params;
    }

    // Doesn't work with disabled entities, yet another example of `Disabled`
    // causing unexpected behavior.
    fn image_bundle(image: Handle<Image>) -> impl Bundle {
        (
            ImageNode { image, ..default() },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                width: Val::Percent(35.0),
                ..default()
            },
        )
    }
}

pub fn reach_target(
    mut commands: Commands,
    target: Single<(Entity, &Transform), With<TargetPosition>>,
    camera: Single<(&Transform, &Stationary), With<Camera>>,
) {
    let (camera_transform, stationary) = camera.into_inner();
    if stationary.0.elapsed_secs() < 1.0 {
        return;
    }

    let (entity, transform) = target.into_inner();
    let dist = camera_transform
        .translation
        .distance_squared(transform.translation);
    if dist < 500.0 {
        commands.entity(entity).despawn();
        commands.set_state(Minigame::None);
    }
}
