use crate::{minigame::prelude::*, prelude::*};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;

pub fn dream_plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<DreamAssets>())
        .add_systems(Startup, spawn_styles)
        .add_systems(OnEnter(GameState::Playing), spawn_variants)
        .add_systems(OnEnter(Minigame::Dream), lock_camera)
        .add_systems(Update, scramble.run_if(in_state(Minigame::Dream)));
}

#[derive(AssetCollection, Resource)]
struct DreamAssets {
    #[asset(path = "sfx/cut.ogg")]
    cut: Handle<AudioSample>,
    //
    #[asset(path = "music/rain.ogg")]
    rain: Handle<AudioSample>,
    #[asset(path = "music/deep.ogg")]
    deep: Handle<AudioSample>,
    #[asset(path = "music/birds.ogg")]
    birds: Handle<AudioSample>,
    #[asset(path = "music/swamp.ogg")]
    swamp: Handle<AudioSample>,
}

fn spawn_styles(mut commands: Commands, mut materials: ResMut<Assets<Glitch>>) {
    commands.spawn((
        PrettyStyle("p"),
        effects![
            PrettyTextMaterial(materials.add(Glitch::default())),
            // wtf
            Scramble {
                speed: ScrambleSpeed::Random(18f32..22f32),
                lifetime: ScrambleLifetime::Always,
            }
        ],
    ));
}

fn spawn_variants(mut commands: Commands, assets: Res<DreamAssets>) {
    let variations = children![(
        VariationSet,
        NotRandom,
        children![
            variation(
                &mut commands,
                Default::default(),
                assets.cut.clone(),
                assets.deep.clone(),
                (
                    AnimationTarget::entity(),
                    DespawnFinished,
                    animations![
                        await_input((
                            FadeIn::default(),
                            pretty!(
                                "|1|Darkness shrouds my body.|1|<1.5> [Where are my \
                                hands?](wave, gray)"
                            )
                        )),
                        await_input((
                            FadeIn::default(),
                            pretty!(
                                "|2|Do [you](red) feel like I feel?|1| Do [you](red) feel me \
                                [scraping](glitch) at [your](red) walls?"
                            )
                        )),
                        // "STATIC ITCH",
                        queue_minigame(Minigame::Typing),
                        system(run_choose_systems),
                    ],
                )
            ),
            variation(
                &mut commands,
                Default::default(),
                assets.cut.clone(),
                assets.rain.clone(),
                (
                    AnimationTarget::entity(),
                    DespawnFinished,
                    animations![
                        await_input((
                            FadeIn::default(),
                            pretty!(
                                "|1|Yes,|0.2| [you](red) are bigger now than \
                                    [you](red) were.|1| \
                                    [I am falling into [you](red)?](wave, gray)"
                            )
                        )),
                        // "NO ESCAPE",
                        queue_minigame(Minigame::Typing),
                        system(run_choose_systems),
                    ],
                )
            ),
            variation(
                &mut commands,
                Default::default(),
                assets.cut.clone(),
                assets.swamp.clone(),
                (
                    PlaybackSettings::default().with_speed(0.5),
                    AnimationTarget::entity(),
                    DespawnFinished,
                    animations![
                        await_finish(pretty!("I am sick of [you](red)|0.5|")),
                        await_finish(pretty!("[wriggling](glitch) in my body|0.5|")),
                        await_finish(pretty!("<0.75>[GET OUT](shake, red)|0.1|")),
                        // "CONSUMING MIND",
                        queue_minigame(Minigame::Typing),
                        system(run_choose_systems),
                    ],
                )
            ),
            variation(
                &mut commands,
                Default::default(),
                assets.cut.clone(),
                assets.birds.clone(),
                (
                    PlaybackSettings::default().with_speed(0.6),
                    AnimationTarget::entity(),
                    DespawnFinished,
                    animations![
                        await_finish((Scramb, pretty!("<0.6>what are you doing here"))),
                        await_finish((Scramb, pretty!("<0.6>this is my code silly"))),
                        // "INEVITABILITY",
                        queue_minigame(Minigame::Typing),
                        system(run_choose_systems),
                    ],
                )
            ),
        ],
    )];

    commands.spawn((
        MinigameRoot,
        DespawnOnExit(GameState::Playing),
        AvailableAfter(6),
        Minigame::Dream,
        variations,
    ));

    fn variation(
        commands: &mut Commands,
        image: Handle<Image>,
        sfx: Handle<AudioSample>,
        music: Handle<AudioSample>,
        bundle: impl Bundle,
    ) -> impl Bundle {
        let mut bundle = Some(bundle);
        let on_start = OnVariationEnable(commands.register_system(
            move |_: In<Entity>,
                  mut commands: Commands,
                  mut opacity: Single<&mut Opacity, With<Fractal>>| {
                opacity.0 = 0.0;
                if let Some(bundle) = bundle.take() {
                    commands.spawn((bundle, SamplePlayer::new(music.clone()).looping()));
                }
            },
        ));
        (Variation, on_start, CutTransition { image, sfx })
    }
}

#[derive(Component)]
struct Scramb;

// idk wtf this is doing damn you pretty text
fn scramble(
    time: Res<Time>,
    mut writer: TextUiWriter,
    entities: Query<Entity, (With<Scramb>, With<Text>)>,
    mut timer: Local<Option<Timer>>,
) {
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.1, TimerMode::Repeating));
    timer.tick(time.delta());
    if timer.just_finished() {
        for entity in entities.iter() {
            let mut i = 0;
            while let Some(mut text) = writer.get_text(entity, i) {
                i += 1;
                *text = text
                    .chars()
                    .map(|c| {
                        if c.is_whitespace() {
                            return c;
                        }
                        let n = (c as u32) % 52;
                        if n < 26 {
                            char::from_u32(65 + n).unwrap()
                        } else {
                            char::from_u32(97 + n - 26).unwrap()
                        }
                    })
                    .collect();
            }
        }
    }
}
