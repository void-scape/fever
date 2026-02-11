use bevy::prelude::*;
use bevy_seedling::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, track_lpf);
}

#[derive(Clone, Copy, Component)]
pub struct Lpf(pub f32);

impl Lpf {
    pub fn distance(d: f32, max: f32) -> Self {
        Self((1.0 - (d / max).clamp(0.0, 1.0)) * 20_000.0 + 100.0)
    }
}

fn track_lpf(
    sample_players: Query<(&SampleEffects, &Lpf)>,
    mut lpf: Query<&mut LowPassNode>,
) -> Result {
    for (effects, param) in &sample_players {
        lpf.get_effect_mut(effects)?.frequency = param.0;
    }
    Ok(())
}
