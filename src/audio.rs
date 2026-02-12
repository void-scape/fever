use crate::animation::AnimationSystems;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use fever_macros::Lerp;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (lpf, playback_speed, linear_volume).after(AnimationSystems::Interpolate),
    );
}

#[derive(Clone, Copy, Component, Lerp)]
pub struct LinearVolume(pub f32);

impl Default for LinearVolume {
    fn default() -> Self {
        Self(1.0)
    }
}

fn linear_volume(
    sample_players: Query<(&SampleEffects, &LinearVolume), Changed<LinearVolume>>,
    mut volume: Query<&mut VolumeNode>,
) -> Result {
    for (effects, param) in &sample_players {
        volume.get_effect_mut(effects)?.volume = Volume::Linear(param.0);
    }
    Ok(())
}

#[derive(Clone, Copy, Component, Lerp)]
pub struct Lpf(pub f32);

impl Default for Lpf {
    fn default() -> Self {
        Self(20_000.0)
    }
}

impl Lpf {
    pub fn distance(d: f32, max: f32) -> Self {
        Self((1.0 - (d / max).clamp(0.0, 1.0)) * 20_000.0 + 100.0)
    }
}

fn lpf(
    sample_players: Query<(&SampleEffects, &Lpf), Changed<Lpf>>,
    mut lpf: Query<&mut LowPassNode>,
) -> Result {
    for (effects, param) in &sample_players {
        lpf.get_effect_mut(effects)?.frequency = param.0;
    }
    Ok(())
}

#[derive(Clone, Copy, Component, Lerp)]
pub struct PlaybackSpeed(pub f64);

impl Default for PlaybackSpeed {
    fn default() -> Self {
        Self(1.0)
    }
}

fn playback_speed(
    mut playbacks: Query<(&mut PlaybackSettings, &PlaybackSpeed), Changed<PlaybackSpeed>>,
) {
    for (mut settings, playback) in playbacks.iter_mut() {
        settings.speed = playback.0;
    }
}

// // Implementation taken from the lovely DaisySP:
// // https://github.com/electro-smith/DaisySP/blob/master/Source/Filters/onepole.h
// #[derive(Component)]
// pub struct LowPass {
//     g: f32,
//     gi: f32,
//     state: f32,
// }
//
// impl LowPass {
//     pub fn new(freq: f32, sample_rate: f32) -> Self {
//         let mut slf = Self {
//             g: 0.0,
//             gi: 0.0,
//             state: 0.0,
//         };
//         slf.set_freq(freq, sample_rate);
//         slf
//     }
//
//     pub fn set_freq(&mut self, freq: f32, sample_rate: f32) {
//         let clipped_freq = (freq / sample_rate).clamp(0.0, 0.497);
//         self.g = (PI * clipped_freq).tan();
//         self.gi = 1.0 / (1.0 + self.g);
//     }
//
//     pub fn process(&mut self, sample: f32) -> f32 {
//         let lp = (self.g * sample + self.state) * self.gi;
//         self.state = self.g * (sample - lp) + lp;
//         lp
//     }
// }
