use crate::minigame::prelude::*;
use crate::prelude::*;
use bevy::{
    color::palettes::css::RED,
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_seedling::prelude::*;

pub fn typing_plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(GameState::Loading).load_collection::<TypingAssets>())
        .add_systems(OnEnter(GameState::Playing), spawn_variants)
        .add_systems(OnEnter(Minigame::Typing), lock_camera)
        .add_systems(OnEnter(Minigame::Typing), force_camera_origin)
        .add_systems(OnExit(Minigame::Typing), unforce_camera_origin)
        .add_systems(
            Update,
            // TODO: feels like shit when you can see the text but cant type
            (
                typing.run_if(in_state(Minigame::Typing).and(in_state(Transition::None))),
                animate_julia.run_if(in_state(Minigame::Typing)),
            ),
        );
}

#[derive(AssetCollection, Resource)]
struct TypingAssets {
    #[asset(path = "images/fractals/star-ship.png")]
    star_ship: Handle<Image>,
    #[asset(path = "images/fractals/odd-julia.png")]
    odd_julia: Handle<Image>,
    #[asset(path = "images/fractals/pl-julia.png")]
    pl_julia: Handle<Image>,
    //
    #[asset(path = "sfx/unhovered.ogg")]
    incorrect: Handle<AudioSample>,
}

fn spawn_variants(mut commands: Commands, assets: Res<TypingAssets>) {
    let text = children![
        target(
            &mut commands,
            "STATIC ITCH",
            (
                Iterations(8.0),
                FractalTexture(assets.star_ship.clone()),
                BurningShip(1),
                CPlane(Vec2::new(0.4888928, 0.08791673)),
            ),
        ),
        target(
            &mut commands,
            "NO ESCAPE",
            (
                Iterations(8.0),
                FractalTexture(assets.pl_julia.clone()),
                BurningShip(1),
                CPlane(Vec2::new(0.803, -1.122)),
            ),
        ),
        target(
            &mut commands,
            "CONSUMING MIND",
            (
                Iterations(4.0),
                FractalTexture(assets.odd_julia.clone()),
                BurningShip(1),
                CPlane(Vec2::new(-0.137, -1.257)),
            ),
        ),
        target(
            &mut commands,
            "INEVITABILITY",
            (
                Iterations(4.0),
                FractalTexture(assets.odd_julia.clone()),
                Exponent(5.0),
                CPlane(Vec2::new(0.667, 0.512)),
            ),
        ),
    ];

    commands.spawn((
        MinigameRoot,
        NotChoosable,
        DespawnOnExit(GameState::Playing),
        Minigame::Typing,
        children![(VariationSet, NotRandom, text)],
    ));

    fn target(
        commands: &mut Commands,
        text: impl Into<String>,
        bundle: impl Bundle,
    ) -> impl Bundle {
        let text = text.into();
        let mut bundle = Some(bundle);
        let on_start = OnVariationEnable(commands.register_system(
            move |root: In<Entity>,
                  mut commands: Commands,
                  fractal: Single<Entity, With<Fractal>>| {
                if let Some(bundle) = bundle.take() {
                    commands.entity(*fractal).insert(bundle);
                }

                commands.entity(*root).with_child((
                    TypingTarget {
                        text: text.clone(),
                        index: 0,
                    },
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
            TimerDuration(5.0),
            ControlsTransition::Keyboard,
        )
    }
}

fn animate_julia(time: Res<Time>, mut cplane: Single<&mut CPlane, With<Fractal>>) {
    cplane.0 += Vec2::from_angle(time.elapsed_secs_wrapped()) * 0.0001;
}

#[derive(Component)]
struct TypingTarget {
    text: String,
    index: usize,
}

fn typing(
    mut commands: Commands,
    mut input: MessageReader<KeyboardInput>,
    target: Single<(&mut TypingTarget, &Children)>,
    mut sections: Query<&mut TextSpan>,
    assets: Res<TypingAssets>,
    text_assets: Res<MinigameAssets>,
) {
    let (mut typing, ui) = target.into_inner();
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
                    SamplePlayer::new(text_assets.presence_glyph.clone())
                        .with_volume(Volume::Linear(0.8)),
                );
            }
        }
    }

    let (typed, rest) = typing.text.split_at(typing.index);
    sections.get_mut(ui[0]).unwrap().0 = typed.to_string();
    sections.get_mut(ui[1]).unwrap().0 = rest.to_string();

    if typing.index >= typing.text.len() {
        commands.run_system_cached(success);
    }
}
