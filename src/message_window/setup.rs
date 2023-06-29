use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*,
    render::view::visibility::RenderLayers, ui::camera_config::UiCameraConfig,
};

#[derive(Component, Debug)]
pub struct MessageWindowCamera;

#[derive(Resource, Default)]
pub struct SetupConfig {
    pub render_layer: u8,
    pub render_order: isize,
}

pub fn setup_camera(mut commands: Commands, config: Res<SetupConfig>) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: config.render_order,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
        RenderLayers::layer(config.render_layer),
        UiCameraConfig { show_ui: false },
        MessageWindowCamera,
    ));
}
