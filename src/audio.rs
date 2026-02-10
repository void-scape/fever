#![allow(unused)]

use bevy::prelude::*;
use bevy_seedling::{configuration::MusicPool, prelude::*};

pub fn music(path: impl Into<String>) -> impl Command {
    let path = path.into();
    move |world: &mut World| {
        let server = world.resource::<AssetServer>();
        let source = server.load(path);
        world.spawn((MusicPool, SamplePlayer::new(source)));
    }
}

pub fn music_volume(path: impl Into<String>, linear_volume: f32) -> impl Command {
    let path = path.into();
    move |world: &mut World| {
        let server = world.resource::<AssetServer>();
        let source = server.load(path);
        world.spawn((
            MusicPool,
            SamplePlayer::new(source).with_volume(Volume::Linear(linear_volume)),
        ));
    }
}

pub fn sfx(path: impl Into<String>) -> impl Command {
    let path = path.into();
    move |world: &mut World| {
        let server = world.resource::<AssetServer>();
        let source = server.load(path);
        world.spawn(SamplePlayer::new(source));
    }
}

pub fn sfx_volume(path: impl Into<String>, linear_volume: f32) -> impl Command {
    let path = path.into();
    move |world: &mut World| {
        let server = world.resource::<AssetServer>();
        let source = server.load(path);
        world.spawn(SamplePlayer::new(source).with_volume(Volume::Linear(linear_volume)));
    }
}
