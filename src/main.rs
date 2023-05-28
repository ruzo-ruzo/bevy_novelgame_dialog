#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;

mod fox_background;
mod message_window;
mod read_script;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(message_window::MessageWindowPlugin)
        .add_plugin(fox_background::FoxBackgroundPlugin)
        .run();
}
