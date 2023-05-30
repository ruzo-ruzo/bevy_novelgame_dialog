use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{color::Color, view::visibility::RenderLayers},
    sprite::Anchor,
    ui::camera_config::UiCameraConfig,
    utils::Duration,
};

mod message_writer;
mod page_scroller;
mod window_controller;

use crate::read_script::*;
use message_writer::*;
use page_scroller::*;
use window_controller::*;

pub struct MessageWindowPlugin;

impl Plugin for MessageWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<BMWScript>()
            .init_asset_loader::<BMWScriptLoader>()
            .init_resource::<MessageWindowConfig>()
            .init_resource::<MessageWindowSetupConfig>()
            .init_resource::<MessageDisplayConfig>()
            .init_resource::<LoadedText>()
            .add_event::<PageScrollEvent>()
            .add_event::<LineFeedEvent>()
            .add_event::<ScrollWaitingEvent>()
            .add_systems(
                Startup,
                (setup_config, setup_message_window.after(setup_config)),
            )
            .add_systems(
                Update,
                (
                    script_on_load,
                    update_message,
                    update_line_height.after(update_message),
                    start_page_scroll,
                    scroll_lines,
                    remove_lines.after(start_page_scroll),
                    trigger_page_scroll,
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct MessageWindowConfig {
    font_handles: Vec<Handle<Font>>,
    background_pict: Handle<Image>,
    layers: RenderLayers,
    current_text_style: TextStyle,
    wait_ps_count: Option<usize>,
    //↓0で全ての行をまとめて改行
    scroll_size: usize,
    scroll_speed: f32,
    wait_reading_time: f32,
    state: MessageWindowState,
}

//他のデータとまとめるとなんかうまく動かなかったのでTimer系列だけ個別にしとく
#[derive(Resource, Default)]
pub struct MessageDisplayConfig {
    char_add_timer: Timer,
}

#[derive(Resource, Default)]
pub struct MessageWindowSetupConfig {
    text_area_origin: Vec2,
    text_area_size: Vec2,
    message_window_origin: Vec2,
    script_path: String,
}

#[derive(Resource, Default)]
pub struct LoadedText {
    pub base_bms: Handle<BMWScript>,
    pub char_list: String,
    pub loading: bool,
}

fn setup_config(
    asset_server: Res<AssetServer>,
    mut config: ResMut<MessageWindowConfig>,
    mut setup_config: ResMut<MessageWindowSetupConfig>,
    mut char_timer_config: ResMut<MessageDisplayConfig>,
    mut loaded_text: ResMut<LoadedText>,
) {
    setup_config.text_area_origin = Vec2::new(-530.0, 70.0);
    setup_config.text_area_size = Vec2::new(1060.0, 140.0);

    // let font_paths = &["oshare.TTF" , "toroman.ttf", "GN-KagakuGothic.ttf", "NotoSansJP-VariableFont_wght.ttf"];
    let font_paths = &[
        "yurumoji.ttf",
        "yinghuayunduoxiaohuzi.ttf",
        "Yomogi-Regular.ttf",
        "NotoSansJP-Black.ttf",
    ];
    config.font_handles = font_paths
        .iter()
        .map(|s| asset_server.load(String::from("fonts/") + s))
        .collect::<Vec<Handle<Font>>>();
    let pict_path = "2d_picture/messageframe/material/messageframe_non_line/message_001.png";
    config.background_pict = asset_server.load(pict_path);
    config.layers = RenderLayers::layer(2);
    config.current_text_style = TextStyle {
        font_size: 27.0,
        color: Color::ANTIQUE_WHITE,
        ..default()
    };
    setup_config.message_window_origin = Vec2::new(0., -200.);
    config.wait_ps_count = None;
    config.scroll_size = 0;
    config.scroll_speed = 40.;
    char_timer_config.char_add_timer = Timer::new(Duration::from_millis(70), TimerMode::Repeating);
    config.wait_reading_time = 1.5;
    config.state = MessageWindowState::Writing;
    setup_config.script_path = "scripts/test.bms".to_string();
    loaded_text.loading = false;
}

fn setup_message_window(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    config: Res<MessageWindowConfig>,
    setup_config: Res<MessageWindowSetupConfig>,
    mut loaded_text: ResMut<LoadedText>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
        config.layers,
        UiCameraConfig { show_ui: false },
    ));
    let mw = commands
        .spawn((
            SpriteBundle {
                texture: config.background_pict.clone(),
                transform: Transform::from_translation(
                    setup_config.message_window_origin.extend(0.0),
                ),
                ..default()
            },
            config.layers,
            MessageWindow,
            CurrentMessageWindow,
        ))
        .id();
    let mta = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::TopLeft,
                    color: Color::WHITE.with_a(0.),
                    custom_size: Some(setup_config.text_area_size),
                    ..default()
                },
                transform: Transform::from_translation(setup_config.text_area_origin.extend(0.0)),
                ..default()
            },
            config.layers,
            MessageTextArea {
                area_size: setup_config.text_area_size,
            },
        ))
        .id();
    commands.entity(mw).add_child(mta);
    loaded_text.base_bms = asset_server.load(&setup_config.script_path);
    loaded_text.loading = true;
}
