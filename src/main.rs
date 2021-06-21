use bevy::{
    prelude::*,
    app::App
};
use bevy_miniquad::{DrawFn, MiniquadPlugin};
use std::sync::Arc;

pub fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource::<DrawFn>(Arc::new(Box::new(draw)))
        .add_plugin(MiniquadPlugin)
        .run();
}

fn draw(_app: &mut App) {
}