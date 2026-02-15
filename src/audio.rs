use crate::prelude::*;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use fever_macros::Lerp;
use std::f32::consts::PI;

pub fn audio_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (lpf, playback_speed, linear_volume).after(AnimationSystems::Interpolate),
    );
}

pub struct SamplerBuilder {
    lpf: Option<Lpf>,
    volume: Option<f32>,
    player: SamplePlayer,
}

impl SamplerBuilder {
    pub fn new(player: SamplePlayer) -> Self {
        Self {
            lpf: None,
            volume: None,
            player,
        }
    }

    pub fn lpf(mut self, lpf: Lpf) -> Self {
        self.lpf = Some(lpf);
        self
    }

    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume);
        self
    }

    pub fn build(self) -> impl Bundle {
        (
            self.player,
            match (self.lpf, self.volume) {
                (Some(lpf), None) => {
                    sample_effects![LowPassNode { frequency: lpf.0 }, VolumeNode::default()]
                }
                (None, Some(vol)) => {
                    sample_effects![LowPassNode::default(), VolumeNode::from_linear(vol)]
                }
                (Some(lpf), Some(vol)) => {
                    sample_effects![
                        LowPassNode { frequency: lpf.0 },
                        VolumeNode::from_linear(vol),
                    ]
                }
                (None, None) => sample_effects![LowPassNode::default(), VolumeNode::default()],
            },
            match self.volume {
                Some(volume) => LinearVolume(volume),
                None => LinearVolume(1.0),
            },
            match self.lpf {
                Some(lpf) => lpf,
                None => Lpf::MAX,
            },
        )
    }
}

#[derive(Clone, Copy, Component, Lerp)]
pub struct LinearVolume(pub f32);

impl Default for LinearVolume {
    fn default() -> Self {
        Self(1.0)
    }
}

pub fn fade_volume(duration: f32, vol: f32) -> impl Bundle {
    (
        Duration(duration),
        Keyframe(LinearVolume(vol)),
        Easing::SineInOut,
    )
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
#[require(LowPass::new(20_000.0, 44_100.0))]
pub struct Lpf(pub f32);

impl Default for Lpf {
    fn default() -> Self {
        Self::MAX
    }
}

impl Lpf {
    pub const MIN: Self = Self(100.0);
    pub const MAX: Self = Self(20_000.0);

    pub fn distance(d: f32, max: f32) -> Self {
        Self((1.0 - (d / max).clamp(0.0, 1.0)) * 20_000.0 + 100.0)
    }
}

fn lpf(
    mut sample_players: Query<(&SampleEffects, &Lpf, &mut LowPass), Changed<Lpf>>,
    mut lpf: Query<&mut LowPassNode>,
) -> Result {
    for (effects, param, mut lp) in sample_players.iter_mut() {
        lpf.get_effect_mut(effects)?.frequency = lp.process(param.0);
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

// Implementation taken from the lovely DaisySP:
// https://github.com/electro-smith/DaisySP/blob/master/Source/Filters/onepole.h
#[derive(Component)]
pub struct LowPass {
    g: f32,
    gi: f32,
    pub state: f32,
}

impl LowPass {
    pub fn new(freq: f32, sample_rate: f32) -> Self {
        let mut slf = Self {
            g: 0.0,
            gi: 0.0,
            state: 0.0,
        };
        slf.set_freq(freq, sample_rate);
        slf
    }

    pub fn set_freq(&mut self, freq: f32, sample_rate: f32) {
        let clipped_freq = (freq / sample_rate).clamp(0.0, 0.497);
        self.g = (PI * clipped_freq).tan();
        self.gi = 1.0 / (1.0 + self.g);
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let lp = (self.g * sample + self.state) * self.gi;
        self.state = self.g * (sample - lp) + lp;
        lp
    }
}
