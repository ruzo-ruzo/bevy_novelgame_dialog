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
pub mod waiting;

use super::setup::SetupConfig;
use crate::read_script::*;

#[derive(Component, Debug)]
pub struct DialogBox {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct TextArea {
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
struct DialogBoxBundle {
    dialog_box: DialogBox,
    state: DialogBoxState,
    waitting: WaitBrakerStyle,
    script: LoadedScript,
    popup_type: PopupType,
}

#[derive(Bundle)]
struct TextAreaBundle {
    text_box: TextArea,
    feeding: FeedingStyle,
    config: TypeTextConfig,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum DialogBoxState {
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
    Auto {
        wait_sec: f32,
    },
    Input {
        icon_entity: Option<Entity>,
        is_icon_moving_to_last: bool,
    },
}

#[derive(Event)]
pub struct OpenWindowEvent {
    pub window_name: String,
    pub font_paths: Vec<String>,
    pub font_size: f32,
    pub font_color: Color,
    pub background_path: String,
    pub dialog_box_entity: Option<Entity>,
    pub position: Vec2,
    pub box_name: String,
    pub popup: PopupType,
    pub typing_timing: TypingTiming,
    pub writing: WritingStyle,
    pub feeding: FeedingStyle,
    pub wait_breaker: WaitBrakerStyle,
    pub script_path: String,
    pub template_path: String,
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
            dialog_box_entity: None,
            position: Vec2::new(0., 0.),
            box_name: "Main Box".to_string(),
            popup: PopupType::Scale { sec: 0.8 },
            typing_timing: TypingTiming::ByChar { sec: 0.07 },
            writing: WritingStyle::Wipe { sec: 0.07 },
            feeding: FeedingStyle::Scroll { size: 0, sec: 40. },
            wait_breaker: WaitBrakerStyle::Auto { wait_sec: 1.5 },
            script_path: "scripts/message.bds".to_string(),
            template_path: "scripts/template.bdt".to_string(),
            main_box_origin: Vec2::new(-600., 80.),
            main_box_size: Vec2::new(1060., 260.),
            main_alignment: TextAlignment::Left,
        }
    }
}
