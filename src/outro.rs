use crate::prelude::*;
use bevy::prelude::*;

pub fn outro_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Outro), exit);
}

fn exit(
    mut commands: Commands,
    camera: Single<Entity, With<CameraTransition>>,
    fractal: Single<Entity, With<Fractal>>,
) {
    commands.entity(*fractal).insert(Opacity(0.0));
    commands
        .entity(*camera)
        .insert(CameraTransitionProgress(0.0));
    commands.spawn((
        AnimationTarget(*camera),
        DespawnFinished,
        animations![
            (Duration(2.0), Keyframe(CameraTransitionProgress(1.0))),
            text_node("Thank you for playing!", 3.0),
            text_node(
                "I could not have made this game without my lovely \
                playtester, Corvus Prudens.",
                6.0
            ),
            text_node("I would love to hear how this game made you feel <3", 5.0),
            text_node("Good luck.", 2.0),
            system(|mut writer: MessageWriter<AppExit>| {
                writer.write(AppExit::Success);
            }),
        ],
    ));
}

fn text_node(text: &'static str, duration: f32) -> impl Bundle {
    #[derive(Component)]
    struct Finished;

    blocking_system(
        move |mut commands: Commands,
              mut entity: Local<Option<Entity>>,
              finished: Query<Entity, With<Finished>>| {
            entity.get_or_insert_with(|| {
                let entity = commands.spawn_empty().id();
                commands
                    .entity(entity)
                    .insert((
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
                            Text::new(text),
                            TextFont::from_font_size(50.0),
                            TextLayout::new_with_justify(Justify::Center),
                            AnimationTarget::entity(),
                            TextColorTween(Color::WHITE.with_alpha(0.0)),
                            animations![
                                (
                                    Duration(1.0),
                                    Keyframe(TextColorTween(Color::WHITE)),
                                    Easing::SineInOut
                                ),
                                Duration((duration - 2.0).max(1.0)),
                                (
                                    Duration(1.0),
                                    Keyframe(TextColorTween(Color::WHITE.with_alpha(0.0))),
                                    Easing::SineInOut
                                ),
                                system(move |mut commands: Commands| {
                                    commands.entity(entity).insert(Finished);
                                }),
                            ]
                        )],
                    ))
                    .id()
            });
            let mut result = false;
            for entity in finished.iter() {
                result = true;
                commands.entity(entity).despawn();
            }
            result
        },
    )
}
