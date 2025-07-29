use bevy::{
    prelude::*,
    render::{camera::ClearColorConfig, view::visibility::RenderLayers},
};

#[derive(Component, Debug)]
pub struct DialogBoxCamera;

#[derive(Resource, Default)]
pub struct SetupConfig {
    pub render_layer: u8,
    pub render_order: isize,
}

pub fn setup_camera(mut commands: Commands, config: Res<SetupConfig>) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: config.render_order,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(config.render_layer.into()),
        DialogBoxCamera,
    ));
}
