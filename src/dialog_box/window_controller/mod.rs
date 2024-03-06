use bevy::{
    prelude::*,
    render::{
        color::Color,
        view::{RenderLayers, Visibility::Hidden},
    },
    sprite::Anchor,
    text::JustifyText,
};

pub mod choice;
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
    pub alignment: JustifyText,
    pub pos_z: f32,
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
    WaitingAction,
    Feeding,
    Pending,
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
pub struct OpenDialogEvent {
    pub dialog_box_name: String,
    pub font_paths: Vec<String>,
    pub font_size: f32,
    pub font_color: Color,
    pub background_path: String,
    pub dialog_box_entity: Option<Entity>,
    pub position: Vec2,
    pub area_name: String,
    pub popup: PopupType,
    pub typing_timing: TypingTiming,
    pub writing: WritingStyle,
    pub feeding: FeedingStyle,
    pub wait_breaker: WaitBrakerStyle,
    pub script_path: String,
    pub template_path: String,
    pub main_box_origin: Vec2,
    pub main_box_size: Vec2,
    pub main_alignment: JustifyText,
    pub template_open_choice: ChoiceBoxConfig,
    pub text_pos_z: f32,
}

impl Default for OpenDialogEvent {
    fn default() -> Self {
        OpenDialogEvent {
            dialog_box_name: "Main Window".to_string(),
            font_paths: vec!["fonts/NotoSans-Black.ttf".to_string()],
            font_size: 27.0,
            font_color: Color::ANTIQUE_WHITE,
            background_path: "texture/ui/text_box.png".to_string(),
            dialog_box_entity: None,
            position: Vec2::new(0., 0.),
            area_name: "Main Area".to_string(),
            popup: PopupType::Scale { sec: 0.8 },
            typing_timing: TypingTiming::ByChar { sec: 0.07 },
            writing: WritingStyle::Wipe { sec: 0.07 },
            feeding: FeedingStyle::Scroll { size: 0, sec: 40. },
            wait_breaker: WaitBrakerStyle::Auto { wait_sec: 1.5 },
            script_path: "scripts/message.bds".to_string(),
            template_path: "scripts/template.bdt".to_string(),
            main_box_origin: Vec2::new(-600., 80.),
            main_box_size: Vec2::new(1060., 260.),
            main_alignment: JustifyText::Left,
            template_open_choice: ChoiceBoxConfig::default(),
            text_pos_z: 1.0,
        }
    }
}

#[derive(Event, Clone)]
pub struct BottunCursoredEvent(pub usize);

#[derive(Event, Clone)]
pub struct BottunClickedEvent(pub usize);

#[derive(Component)]
pub struct ChoiceBoxState {
    main_dialog_box: Entity,
    open_dialog_event: OpenDialogEvent,
    button_entities: Vec<Entity>,
    target_list: Vec<(String, String)>,
    button_box_origin: Vec2,
    button_box_size: Vec2,
}

#[derive(Component, Clone)]
pub struct ChoiceBoxConfig {
    pub background_entities: Option<Entity>,
    pub button_entities: Vec<Entity>,
    pub main_alignment: JustifyText,
    pub dialog_box_name: String,
    pub button_box_origin: Vec2,
    pub button_box_size: Vec2,
    pub popup: PopupType,
}

impl Default for ChoiceBoxConfig {
    fn default() -> Self {
        ChoiceBoxConfig {
            background_entities: None,
            button_entities: Vec::new(),
            main_alignment: JustifyText::Center,
            dialog_box_name: "Choice Window".to_string(),
            button_box_origin: Vec2::new(-60., 20.),
            button_box_size: Vec2::new(600., 80.),
            popup: PopupType::Scale { sec: 0.8 },
        }
    }
}
