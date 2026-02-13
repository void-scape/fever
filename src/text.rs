use crate::animation::*;
use crate::minigame::MinigameAssets;
use bevy::prelude::*;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_observer(advance).add_observer(glyph);
}

fn text_node(text: impl Bundle) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(500),
        children![(
            Node {
                width: percent(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            text,
            TextFont::from_font_size(50.0),
            TextLayout::new_with_justify(Justify::Center),
            Typewriter::new(15.0),
        )],
    )
}

pub fn await_finish(text: impl Bundle) -> impl Bundle {
    let mut text = Some(text);
    blocking_system(
        move |mut commands: Commands,
              advance: Query<Entity, With<Advance>>,
              mut entity: Local<Option<Entity>>,
              input: Res<ButtonInput<KeyCode>>,
              typewriters: Query<Entity, With<Typewriter>>| {
            if let Some(text) = text.take() {
                *entity = Some(commands.spawn(text_node(text)).id());
            }

            if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
                for entity in typewriters.iter() {
                    commands.entity(entity).insert(FinishTypewriter);
                }
                return false;
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

pub fn await_input(text: impl Bundle) -> impl Bundle {
    let mut text = Some(text);
    blocking_system(
        move |mut commands: Commands,
              advance: Query<Entity, With<Advance>>,
              input: Res<ButtonInput<KeyCode>>,
              mut awaiting_input: Local<bool>,
              mut entity: Local<Option<Entity>>,
              typewriters: Query<Entity, With<Typewriter>>| {
            if let Some(text) = text.take() {
                *entity = Some(commands.spawn(text_node(text)).id());
            }

            if !*awaiting_input {
                if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
                    for entity in typewriters.iter() {
                        commands.entity(entity).insert(FinishTypewriter);
                    }
                    *awaiting_input = true;
                    return false;
                }

                for entity in advance.iter() {
                    *awaiting_input = true;
                    commands.entity(entity).despawn();
                }
                false
            } else {
                let result = input
                    .get_just_pressed()
                    .any(|k| *k == KeyCode::Space || *k == KeyCode::Enter);
                if result && let Some(entity) = *entity {
                    commands.entity(entity).despawn();
                }
                if result {
                    for entity in advance.iter() {
                        commands.entity(entity).despawn();
                    }
                }
                result
            }
        },
    )
}

#[derive(Component)]
struct Advance;

fn advance(_: On<TypewriterFinished>, mut commands: Commands) {
    commands.spawn(Advance);
}

fn glyph(revealed: On<Revealed<Char>>, mut commands: Commands, assets: Res<MinigameAssets>) {
    if revealed.event().text != " " {
        commands.spawn((
            SamplePlayer::new(assets.glyph.clone()).with_volume(Volume::Linear(0.8)),
            RandomPitch::new(0.05),
        ));
    }
}
