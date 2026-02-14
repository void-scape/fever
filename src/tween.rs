use crate::prelude::*;
use bevy::prelude::*;
use fever_macros::Lerp;

pub fn tween_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (image_color, ui_translation, text_color, translation_2d)
            .after(AnimationSystems::Interpolate),
    );
}

#[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct Translation2D(pub Vec2);

fn translation_2d(
    mut translations: Query<(&mut Transform, &Translation2D), Changed<Translation2D>>,
) {
    for (mut transform, translation) in translations.iter_mut() {
        transform.translation.x = translation.x;
        transform.translation.y = translation.y;
    }
}

#[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct TextColorTween(pub Color);

fn text_color(mut colors: Query<(&mut TextColor, &TextColorTween), Changed<TextColorTween>>) {
    for (mut color, tween) in colors.iter_mut() {
        color.0 = tween.0;
    }
}

#[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct ImageColor(pub Color);

fn image_color(mut nodes: Query<(&mut ImageNode, &ImageColor), Changed<ImageColor>>) {
    for (mut node, color) in nodes.iter_mut() {
        node.color = color.0;
    }
}

#[derive(Default, Clone, Copy, Component, Lerp, Deref, DerefMut)]
pub struct UiTranslationPx(pub Vec2);

fn ui_translation(
    mut nodes: Query<(&mut UiTransform, &UiTranslationPx), Changed<UiTranslationPx>>,
) {
    for (mut node, t) in nodes.iter_mut() {
        node.translation = Val2::px(t.x, t.y);
    }
}
