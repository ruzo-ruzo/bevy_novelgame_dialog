use bevy::{
    prelude::*,
    render::{
        color::Color,
        view::{RenderLayers, Visibility::Hidden},
    },
    sprite::Anchor,
    text::TextAlignment,
};

pub mod popup;
pub mod sinkdown;

use super::setup::SetupConfig;
use crate::read_script::*;

#[derive(Component, Debug)]
pub struct MessageWindow {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct TextBox {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct Current;

#[derive(Component, Debug)]
pub struct TypeTextConfig {
    pub fonts: Vec<Handle<Font>>,
    pub text_style: TextStyle,
    pub writing: WritingStyle,
    pub typing_timing: TypingTiming,
    pub layer: RenderLayers,
    pub alignment: TextAlignment,
}

#[derive(Bundle)]
struct MessageWindowBundle {
    message_window: MessageWindow,
    state: WindowState,
    waitting: WaitBrakerStyle,
    script: LoadedScript,
    popup_type: PopupType,
}

#[derive(Bundle)]
struct TextBoxBundle {
    text_box: TextBox,
    feeding: FeedingStyle,
    config: TypeTextConfig,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum WindowState {
    Preparing,
    PoppingUp,
    Typing,
    Waiting,
    Feeding,
    SinkingDown,
    Fixed,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum PopupType {
    Scale { sec: f32 },
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
pub enum SinkDownType {
    #[default]
    Fix,
    Scale {
        sec: f32,
    },
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum TypingTiming {
    ByChar { sec: f32 },
    ByLine { sec: f32 },
    ByPage,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum WritingStyle {
    Wipe { sec: f32 },
    Put,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum FeedingStyle {
    Scroll { size: usize, sec: f32 },
    // Fade,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum WaitBrakerStyle {
    Auto { wait_sec: f32 },
    // Input,
    // Fix,
}

#[derive(Event)]
pub struct OpenWindowEvent {
    pub window_name: String,
    pub font_paths: Vec<String>,
    pub font_size: f32,
    pub font_color: Color,
    pub background_path: String,
    pub position: Vec2,
    pub box_name: String,
    pub popup: PopupType,
    pub typing_timing: TypingTiming,
    pub writing: WritingStyle,
    pub feeding: FeedingStyle,
    pub wait_breaker: WaitBrakerStyle,
    pub script_path: String,
    pub main_box_origin: Vec2,
    pub main_box_size: Vec2,
    pub main_alignment: TextAlignment,
}

impl Default for OpenWindowEvent {
    fn default() -> Self {
        OpenWindowEvent {
            window_name: "Main Window".to_string(),
            font_paths: vec!["fonts/NotoSans-Black.ttf".to_string()],
            font_size: 27.0,
            font_color: Color::ANTIQUE_WHITE,
            background_path: "texture/ui/text_box.png".to_string(),
            position: Vec2::new(0., 0.),
            box_name: "Main Box".to_string(),
            popup: PopupType::Scale { sec: 0.8 },
            typing_timing: TypingTiming::ByChar { sec: 0.07 },
            writing: WritingStyle::Wipe { sec: 0.07 },
            feeding: FeedingStyle::Scroll { size: 0, sec: 40. },
            wait_breaker: WaitBrakerStyle::Auto { wait_sec: 1.5 },
            script_path: "scripts/message.bms".to_string(),
            main_box_origin: Vec2::new(-600., 80.),
            main_box_size: Vec2::new(1060., 260.),
            main_alignment: TextAlignment::Left,
        }
    }
}

pub fn open_window(
    mut commands: Commands,
    mut mw_query: Query<Entity, With<Current>>,
    mut ow_event: EventReader<OpenWindowEvent>,
    asset_server: Res<AssetServer>,
    setup_config: Res<SetupConfig>,
) {
    for window_config in &mut ow_event {
        let mwb = MessageWindowBundle {
            message_window: MessageWindow {
                name: window_config.window_name.clone(),
            },
            state: WindowState::Preparing,
            waitting: window_config.wait_breaker,
            script: LoadedScript {
                bms_handle: asset_server.load(window_config.script_path.clone()),
                order_list: None,
            },
            popup_type: window_config.popup,
        };
        let mw_spirte = SpriteBundle {
            texture: asset_server.load(window_config.background_path.clone()),
            transform: Transform::from_translation(window_config.position.extend(0.0)),
            visibility: Hidden,
            ..default()
        };
        let tbb = TextBoxBundle {
            text_box: TextBox {
                name: window_config.box_name.clone(),
            },
            feeding: window_config.feeding,
            config: TypeTextConfig {
                fonts: window_config
                    .font_paths
                    .iter()
                    .map(|s| asset_server.load(s))
                    .collect(),
                text_style: TextStyle {
                    font_size: window_config.font_size,
                    color: window_config.font_color,
                    ..default()
                },
                writing: window_config.writing,
                typing_timing: window_config.typing_timing,
                layer: RenderLayers::layer(setup_config.render_layer),
                alignment: window_config.main_alignment,
            },
        };
        let tb_sprite = SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::TopLeft,
                color: Color::WHITE.with_a(0.),
                custom_size: Some(window_config.main_box_size),
                ..default()
            },
            transform: Transform::from_translation(window_config.main_box_origin.extend(0.0)),
            ..default()
        };
        for entity in &mut mw_query {
            commands.entity(entity).remove::<Current>();
        }
        let layer = RenderLayers::layer(setup_config.render_layer);
        let mw = commands.spawn((mwb, mw_spirte, layer, Current)).id();
        let tb = commands.spawn((tbb, tb_sprite, layer, Current)).id();
        commands.entity(mw).add_child(tb);
    }
}
