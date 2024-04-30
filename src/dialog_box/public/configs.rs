use super::components::*;
use bevy::{
    prelude::*,
    render::{color::Color, view::RenderLayers},
    sprite::Anchor,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectVector {
    Vertical,
    Horizon,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AlignVertical {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AlignHorizon {
    Left,
    Center,
    Right,
}

#[derive(Clone)]
pub struct FontConfig {
    pub path: String,
    pub kerning: f32,
    pub size: f32,
}

impl Default for FontConfig {
    fn default() -> Self {
        FontConfig {
            path: "fonts/FiraMono-Regular.ttf".to_string(),
            kerning: 0.0,
            size: 1.0,
        }
    }
}

#[derive(Component)]
pub struct TypeTextConfig {
    pub fonts: Vec<Handle<Font>>,
    pub kerning_by_fonts: Vec<f32>,
    pub size_by_fonts: Vec<f32>,
    pub text_style: TextStyle,
    pub writing: WritingStyle,
    pub typing_timing: TypingTiming,
    pub layer: RenderLayers,
    pub horizon_alignment: AlignHorizon,
    pub vertical_alignment: AlignVertical,
    pub pos_z: f32,
}

#[derive(Clone)]
pub struct TextAreaConfig {
    pub area_name: String,
    pub area_origin: Vec2,
    pub area_size: Vec2,
    pub horizon_alignment: AlignHorizon,
    pub vertical_alignment: AlignVertical,
    pub feeding: FeedingStyle,
    pub typing_timing: TypingTiming,
    pub writing: WritingStyle,
    pub font_sets: Vec<FontConfig>,
    pub text_base_size: f32,
    pub font_color: Color,
    pub text_pos_z: f32,
}

impl Default for TextAreaConfig {
    fn default() -> Self {
        TextAreaConfig {
            area_name: "Main Area".to_string(),
            area_origin: Vec2::new(-600., 80.),
            area_size: Vec2::new(1060., 260.),
            horizon_alignment: AlignHorizon::Left,
            vertical_alignment: AlignVertical::Top,
            feeding: FeedingStyle::Scroll { size: 0, sec: 40. },
            typing_timing: TypingTiming::ByChar { sec: 0.07 },
            writing: WritingStyle::Wipe { sec: 0.07 },
            font_sets: vec![FontConfig::default()],
            text_base_size: 27.0,
            font_color: Color::ANTIQUE_WHITE,
            text_pos_z: 1.0,
        }
    }
}

#[derive(Component, Clone)]
pub struct ChoiceBoxConfig {
    pub choice_box_name: String,
    pub button_text_areas: Vec<TextAreaConfig>,
    pub popup: PopupType,
    pub sinkdown: SinkDownType,
    pub wait_to_sink: f32,
    pub select_vector: SelectVector,
    pub background_scaling_per_button: Vec2,
    pub background_scaling_anchor: Anchor,
}

impl Default for ChoiceBoxConfig {
    fn default() -> Self {
        let basic_text_area = TextAreaConfig {
            writing: WritingStyle::Put,
            typing_timing: TypingTiming::ByPage,
            horizon_alignment: AlignHorizon::Center,
            vertical_alignment: AlignVertical::Center,
            ..default()
        };
        ChoiceBoxConfig {
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
            choice_box_name: "Choice Box".to_string(),
            popup: PopupType::Scale { sec: 0.8 },
            sinkdown: SinkDownType::Scale { sec: 0.8 },
            wait_to_sink: 0.0,
            select_vector: SelectVector::Vertical,
            background_scaling_per_button: Vec2::new(0., 100.),
            background_scaling_anchor: Anchor::TopLeft,
        }
    }
}
