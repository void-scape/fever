use crate::{
    audio::Lpf,
    camera::lock_camera,
    fractal::{BurningShip, CPlane, Exponent, Fractal, FractalTexture, Mandelbrot, Zoom},
    minigame::{
        Description, Minigame, MinigameRoot, NotRandom, OnVariationEnable, StartTimer, Variation,
        VariationSet,
    },
    state::GameState,
};
use bevy::{
    color::palettes::css::RED,
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use bevy_seedling::prelude::*;
use rand::Rng;

pub fn plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<TypingAssets>())
        .add_systems(OnEnter(GameState::Playing), init_typing)
        .add_systems(OnEnter(Minigame::Typing), lock_camera)
        .add_systems(Update, typing.run_if(in_state(Minigame::Typing)));
}

#[derive(AssetCollection, Resource)]
struct TypingAssets {
    #[asset(path = "images/fractals/star-ship.png")]
    star_ship: Handle<Image>,

    // TODO: I already used these weird julia sets in the intro... make more?
    #[asset(path = "images/fractals/odd-julia.png")]
    odd_julia: Handle<Image>,
    #[asset(path = "images/fractals/pl-julia.png")]
    pl_julia: Handle<Image>,
    #[asset(path = "images/fractals/last-breath.png")]
    last_breath: Handle<Image>,
    //
    #[asset(path = "sfx/hovered.ogg")]
    correct: Handle<AudioSample>,
    #[asset(path = "sfx/unhovered.ogg")]
    incorrect: Handle<AudioSample>,
    //
    #[asset(path = "music/landscape.ogg")]
    landscape: Handle<AudioSample>,
    #[asset(path = "music/rain.ogg")]
    rain: Handle<AudioSample>,
    #[asset(path = "music/deep.ogg")]
    deep: Handle<AudioSample>,
}

#[derive(Component)]
struct TypingTarget {
    text: String,
    index: usize,
}

#[derive(Component)]
struct TypingText;

fn init_typing(
    mut commands: Commands,
    assets: Res<TypingAssets>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let text = children![
        target(
            &mut commands,
            "SYSTEM OVERRIDE",
            assets.landscape.clone(),
            assets.star_ship.clone(),
            &mut rng,
            None,
        ),
        target(
            &mut commands,
            "ACCESS GRANTED",
            assets.landscape.clone(),
            assets.star_ship.clone(),
            &mut rng,
            None,
        ),
        target(
            &mut commands,
            "REGRET",
            assets.rain.clone(),
            assets.star_ship.clone(),
            &mut rng,
            Some(assets.odd_julia.clone()),
        ),
        target(
            &mut commands,
            "NO ESCAPE",
            assets.deep.clone(),
            assets.star_ship.clone(),
            &mut rng,
            Some(assets.pl_julia.clone()),
        ),
        target(
            &mut commands,
            "INFINITY",
            assets.rain.clone(),
            assets.star_ship.clone(),
            &mut rng,
            Some(assets.last_breath.clone()),
        ),
    ];

    commands.spawn((
        MinigameRoot,
        DespawnOnExit(GameState::Playing),
        Minigame::Typing,
        Description::Keyboard,
        children![(VariationSet, NotRandom, text)],
    ));

    fn target(
        commands: &mut Commands,
        text: impl Into<String>,
        song: Handle<AudioSample>,
        texture: Handle<Image>,
        _rng: &mut impl Rng,
        image: Option<Handle<Image>>,
    ) -> impl Bundle {
        let text = text.into();
        let t = text.clone();
        let on_start = OnVariationEnable(commands.register_system(
            move |_: In<Entity>, mut commands: Commands, fractal: Single<Entity, With<Fractal>>| {
                if let Some(image) = image.clone() {
                    commands.entity(*fractal).insert((
                        FractalTexture(image),
                        Mandelbrot(1),
                        Exponent(6.0),
                        Zoom(1.25),
                    ));
                } else {
                    commands.entity(*fractal).insert((
                        FractalTexture(texture.clone()),
                        BurningShip(1),
                        CPlane(Vec2::new(0.4888928, 0.08791673)),
                    ));
                }

                commands.spawn((
                    DespawnOnExit(Minigame::EnterWipe),
                    TypingText,
                    Text2d::default(),
                    children![
                        (
                            TextSpan::default(),
                            TextColor(RED.into()),
                            TextFont::from_font_size(80.0)
                        ),
                        (TextSpan::new(text.clone()), TextFont::from_font_size(80.0)),
                    ],
                ));
            },
        ));

        (
            Variation,
            on_start,
            StartTimer(5.0),
            TypingTarget { text: t, index: 0 },
            SamplePlayer::new(song)
                .with_volume(Volume::Linear(0.8))
                .looping(),
            sample_effects![LowPassNode {
                frequency: 20_000.0
            }],
            Lpf(20_000.0),
        )
    }
}

fn typing(
    mut commands: Commands,
    mut input: MessageReader<KeyboardInput>,
    target: Single<(&mut TypingTarget, &mut Lpf)>,
    ui: Single<&Children, With<TypingText>>,
    mut sections: Query<&mut TextSpan>,
    assets: Res<TypingAssets>,
) {
    let (mut typing, mut lpf) = target.into_inner();
    let chars = typing.text.chars().collect::<Vec<_>>();
    for event in input.read() {
        if event.state.is_pressed()
            && let Key::Character(s) = &event.logical_key
            && let Some(c) = s.chars().next()
            && typing.index < chars.len()
        {
            let expected = chars[typing.index];
            if c.eq_ignore_ascii_case(&expected) {
                typing.index += 1;
                while typing.index < chars.len() && chars[typing.index] == ' ' {
                    typing.index += 1;
                }
                commands.spawn(
                    SamplePlayer::new(assets.incorrect.clone()).with_volume(Volume::Linear(0.8)),
                );
            } else {
                commands.spawn(
                    SamplePlayer::new(assets.correct.clone()).with_volume(Volume::Linear(0.8)),
                );
            }
        }
    }

    let remaining = typing.text.len().saturating_sub(typing.index);
    *lpf = Lpf::distance(remaining as f32 * 1.5, typing.text.len() as f32);

    let (typed, rest) = typing.text.split_at(typing.index);
    sections.get_mut(ui[0]).unwrap().0 = typed.to_string();
    sections.get_mut(ui[1]).unwrap().0 = rest.to_string();

    if typing.index >= typing.text.len() {
        commands.set_state(Minigame::Success);
    }
}
