use crate::prelude::*;
use bevy::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;

pub fn text_plugin(app: &mut App) {
    app.init_resource::<TextGlyphSfx>()
        .add_observer(advance)
        .add_observer(glyph)
        .add_systems(OnEnter(GameState::Intro), narrator_glyph)
        .add_systems(OnEnter(GameState::PhaseOne), narrator_glyph);
}

fn text_node(_font: Handle<Font>, text: impl Bundle) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        GlobalZIndex(500),
        children![(
            Node {
                width: percent(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            text,
            TextFont {
                font_size: 50.0,
                // font,
                ..Default::default()
            },
            TextLayout::new_with_justify(Justify::Center),
            Typewriter::new(15.0),
        )],
    )
}

pub fn unskippable(text: impl Bundle) -> impl Bundle {
    let mut text = Some(text);
    blocking_system(
        move |mut commands: Commands,
              advance: Query<Entity, With<Advance>>,
              mut entity: Local<Option<Entity>>,
              assets: Res<MinigameAssets>| {
            if let Some(text) = text.take() {
                *entity = Some(commands.spawn(text_node(assets.font.clone(), text)).id());
            }
            let result = !advance.is_empty();
            for entity in advance.iter() {
                commands.entity(entity).despawn();
            }
            if result && let Some(entity) = *entity {
                commands.entity(entity).despawn();
            }
            result
        },
    )
}

#[derive(Component)]
struct Advance;

fn advance(_: On<TypewriterFinished>, mut commands: Commands) {
    commands.spawn(Advance);
}

#[derive(Default, Resource)]
pub struct TextGlyphSfx(pub Handle<AudioSample>);

pub fn narrator_glyph(mut glyph: ResMut<TextGlyphSfx>, assets: Res<MinigameAssets>) {
    glyph.0 = assets.narrator_glyph.clone();
}

pub fn presence_glyph(mut glyph: ResMut<TextGlyphSfx>, assets: Res<MinigameAssets>) {
    glyph.0 = assets.presence_glyph.clone();
}

fn glyph(revealed: On<Revealed<Char>>, mut commands: Commands, glyph: Res<TextGlyphSfx>) {
    if revealed.event().text != " " {
        commands.spawn((
            SamplePlayer::new(glyph.0.clone()).with_volume(Volume::Linear(0.8)),
            RandomPitch::new(0.05),
        ));
    }
}
