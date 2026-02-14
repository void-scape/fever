use crate::{minigame::prelude::*, prelude::*};
use bevy::prelude::*;
use rand::seq::IteratorRandom;
use std::collections::VecDeque;

#[cfg(feature = "dev")]
const RANDOM: bool = true;
#[cfg(not(feature = "dev"))]
const RANDOM: bool = true;

pub fn choose_plugin(app: &mut App) {
    app.init_resource::<ChooseQueue>();
}

pub fn run_choose_systems(world: &mut World) {
    _ = world.run_system_cached(clean_chosen);
    _ = world.run_system_cached(clean_variation_sets);
    _ = world.run_system_cached(clean_minigame_roots);
    _ = world.run_system_cached(choose_root);
    _ = world.run_system_cached(choose_variation);
    _ = world.run_system_cached(available_after);
}

#[derive(Component)]
pub struct AvailableAfter(pub usize);

fn available_after(mut commands: Commands, mut available: Query<(Entity, &mut AvailableAfter)>) {
    for (entity, mut after) in available.iter_mut() {
        after.0 = after.0.saturating_sub(1);
        if after.0 == 0 {
            commands.entity(entity).remove::<AvailableAfter>();
        }
    }
}

#[derive(Component)]
pub struct NotChoosable;

#[derive(Default, Resource, Deref, DerefMut)]
pub struct ChooseQueue(pub VecDeque<Minigame>);

pub fn queue_minigame(minigame: Minigame) -> impl Bundle {
    system(move |mut queue: ResMut<ChooseQueue>| {
        queue.push_back(minigame);
    })
}

pub fn enable_variation(
    mut commands: Commands,
    state: Single<&Minigame, With<ChosenRoot>>,
    variation: Single<
        (Entity, Option<&TimerDuration>, Option<&OnVariationEnable>),
        With<ChosenVariation>,
    >,
    fractal: Single<Entity, With<Fractal>>,
    camera: Single<Entity, With<Camera>>,
) {
    commands.entity(*fractal).insert(ResetFractal);
    commands
        .entity(*camera)
        .insert(MovementSensitivity::default());

    let (variation, timer, on_enable) = variation.into_inner();
    commands
        .entity(variation)
        .remove::<ChildOf>()
        .insert(DespawnOnExit(**state));

    if let Some(on_enable) = on_enable {
        commands.run_system_with(on_enable.0, variation);
    }
    if let Some(timer) = timer {
        commands.spawn((
            DespawnOnEnter(Transition::Enter),
            DespawnOnExit(**state),
            GameTimer(Timer::from_seconds(timer.0, TimerMode::Once)),
            Text::default(),
            TextFont::from_font_size(40.0),
            TextColor::default(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            },
        ));
    }
}

#[derive(Component)]
struct LastChosen;

#[derive(Component)]
pub struct ChosenRoot;

fn choose_root(
    mut commands: Commands,
    roots: Query<(Entity, &Minigame), With<MinigameRoot>>,
    available: Query<
        Entity,
        (
            Without<LastChosen>,
            With<MinigameRoot>,
            Without<AvailableAfter>,
            Without<NotChoosable>,
        ),
    >,
    last_chosen: Query<
        Entity,
        (
            With<LastChosen>,
            With<MinigameRoot>,
            Without<AvailableAfter>,
            Without<NotChoosable>,
        ),
    >,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut queue: ResMut<ChooseQueue>,
) {
    let next = if let Some(root) = queue.pop_front()
        && let Some((next, _)) = roots.iter().find(|(_, r)| **r == root)
    {
        Some(next)
    } else if RANDOM {
        if available.is_empty() && !last_chosen.is_empty() {
            last_chosen.iter().choose(&mut rng)
        } else {
            available.iter().choose(&mut rng)
        }
    } else {
        available.iter().next()
    };
    for entity in last_chosen.iter() {
        commands.entity(entity).remove::<LastChosen>();
    }
    if let Some(entity) = next {
        commands.entity(entity).insert((LastChosen, ChosenRoot));
    } else {
        commands.set_state(GameState::Outro);
    }
}

#[derive(Component)]
pub struct ChosenVariation;

fn choose_variation(
    mut commands: Commands,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    children: Single<&Children, (With<MinigameRoot>, With<ChosenRoot>)>,
    sets: Query<(&Children, Has<NotRandom>), With<VariationSet>>,
    variations: Query<Entity, With<Variation>>,
) {
    let (set, not_random) = sets.iter_many(*children).next().unwrap();
    let entity = if RANDOM && !not_random {
        variations.iter_many(set).choose(&mut rng).unwrap()
    } else {
        variations.iter_many(set).next().unwrap()
    };
    commands.entity(entity).insert(ChosenVariation);
    commands.set_state(Transition::Enter);
}

fn clean_chosen(
    mut commands: Commands,
    roots: Query<Entity, With<ChosenRoot>>,
    variations: Query<Entity, With<ChosenVariation>>,
) {
    for entity in roots.iter() {
        commands.entity(entity).remove::<ChosenRoot>();
    }
    for entity in variations.iter() {
        commands.entity(entity).remove::<ChosenVariation>();
    }
}

fn clean_variation_sets(
    mut commands: Commands,
    sets: Query<Entity, (With<VariationSet>, Without<Children>)>,
) {
    for entity in sets.iter() {
        commands.entity(entity).despawn();
    }
}

fn clean_minigame_roots(
    mut commands: Commands,
    roots: Query<Entity, (With<MinigameRoot>, Without<Children>)>,
) {
    for entity in roots.iter() {
        commands.entity(entity).despawn();
    }
}
