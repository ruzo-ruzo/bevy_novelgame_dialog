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
use crate::dialog_box::OpenDialogEvent;
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

#[derive(Component)]
pub struct Instant;

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
    state: DialogBoxPhase,
    waitting: WaitBrakerStyle,
    script: LoadedScript,
    popup_type: PopupType,
}

#[derive(Bundle)]
struct TextAreaBundle {
    text_area: TextArea,
    feeding: FeedingStyle,
    config: TypeTextConfig,
}

#[derive(Clone)]
pub struct TextAreaConfig {
    pub area_name: String,
    pub area_origin: Vec2,
    pub area_size: Vec2,
    pub main_alignment: JustifyText,
    pub feeding: FeedingStyle,
    pub typing_timing: TypingTiming,
    pub writing: WritingStyle,
    pub font_paths: Vec<String>,
    pub font_size: f32,
    pub font_color: Color,
    pub text_pos_z: f32,
}

impl Default for TextAreaConfig {
    fn default() -> Self {
        TextAreaConfig {
            area_name: "Main Area".to_string(),
            area_origin: Vec2::new(-600., 80.),
            area_size: Vec2::new(1060., 260.),
            main_alignment: JustifyText::Left,
            feeding: FeedingStyle::Scroll { size: 0, sec: 40. },
            typing_timing: TypingTiming::ByChar { sec: 0.07 },
            writing: WritingStyle::Wipe { sec: 0.07 },
            font_paths: vec!["fonts/FiraMono-Regular.ttf".to_string()],
            font_size: 27.0,
            font_color: Color::ANTIQUE_WHITE,
            text_pos_z: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum DialogBoxPhase {
    Preparing,
    PoppingUp,
    Typing,
    WaitingAction,
    Feeding,
    Fixed,
    SinkingDown,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectVector {
    Vertical,
    Horizon,
}

#[derive(Component)]
pub struct ChoiceBoxState {
    main_dialog_box: Entity,
    choice_box_entity: Option<Entity>,
    text_area_names: Vec<String>,
    button_entities: Vec<Entity>,
    target_list: Vec<(String, String)>,
    select_vector: SelectVector,
    sinkdown: SinkDownType,
}

#[derive(Component, Clone)]
pub struct ChoiceBoxConfig {
    pub background_entity: Option<Entity>,
    pub button_entities: Vec<Entity>,
    pub button_text_areas: Vec<TextAreaConfig>,
    pub dialog_box_name: String,
    pub popup: PopupType,
    pub sinkdown: SinkDownType,
    pub select_vector: SelectVector,
    pub background_scaling_per_button: Vec2,
    pub background_scaling_anchor: Anchor,
}

impl Default for ChoiceBoxConfig {
    fn default() -> Self {
        let basic_text_area = TextAreaConfig {
            writing: WritingStyle::Put,
            typing_timing: TypingTiming::ByPage,
            main_alignment: JustifyText::Center,
            ..default()
        };
        ChoiceBoxConfig {
            background_entity: None,
            button_entities: Vec::new(),
            button_text_areas: vec![
                TextAreaConfig {
                    area_name: "Button Area 01".to_string(),
                    area_origin: Vec2::new(0., 100.),
                    ..basic_text_area.clone()
                },
                TextAreaConfig {
                    area_name: "Button Area 02".to_string(),
                    area_origin: Vec2::new(0., 0.),
                    ..basic_text_area.clone()
                },
                TextAreaConfig {
                    area_name: "Button Area 03".to_string(),
                    area_origin: Vec2::new(0., -100.),
                    ..basic_text_area
                },
            ],
            dialog_box_name: "Choice Box".to_string(),
            popup: PopupType::Scale { sec: 0.8 },
            sinkdown: SinkDownType::Scale { sec: 0.8 },
            select_vector: SelectVector::Vertical,
            background_scaling_per_button: Vec2::new(0., 100.),
            background_scaling_anchor: Anchor::TopLeft,
        }
    }
}
