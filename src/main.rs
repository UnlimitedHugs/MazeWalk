mod rendering;

use bevy::{
    prelude::*,
    app::App
};
use bevy_miniquad::{MiniquadPlugin};
use rendering::RenderingPlugin;

pub fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(MiniquadPlugin)
        .add_plugin(RenderingPlugin)
        .run();
}