use super::params::*;
use bevy::{
    color::palettes::css as CssColor, prelude::*, render::view::RenderLayers, sprite::Anchor,
};
use std::collections::HashMap;

#[derive(Component)]
pub struct TypeTextConfig {
    pub text_fonts: Vec<TextFont>,
    pub kerning_by_regulars: HashMap<String, f32>,
    pub size_by_regulars: HashMap<String, f32>,
    pub text_color: TextColor,
    pub writing: WritingStyle,
    pub base_size: f32,
    pub typing_timing: TypingTiming,
    pub layer: RenderLayers,
    pub horizon_alignment: AlignHorizon,
    pub vertical_alignment: AlignVertical,
    pub monospace: bool,
    pub pos_z: f32,
}

#[derive(Clone)]
pub struct CharConfig {
    pub font_settings: Vec<FontSettings>,
    pub kerning_by_regulars: HashMap<String, f32>,
    pub size_by_regulars: HashMap<String, f32>,
    pub text_base_size: f32,
    pub font_color: TextColor,
}

impl Default for CharConfig {
    fn default() -> Self {
        CharConfig {
            font_settings: vec![FontSettings::default()],
            kerning_by_regulars: HashMap::default(),
            size_by_regulars: HashMap::default(),
            text_base_size: 27.0,
            font_color: CssColor::ANTIQUE_WHITE.into(),
        }
    }
}

#[derive(Clone)]
pub struct TextAreaConfig {
    pub area_name: String,
    pub area_origin: Vec2,
    pub area_size: Vec2,
    pub horizon_alignment: AlignHorizon,
    pub vertical_alignment: AlignVertical,
    pub monospace: bool,
    pub text_config: CharConfig,
    pub feeding: FeedingStyle,
    pub typing_timing: TypingTiming,
    pub writing: WritingStyle,
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
            monospace: false,
            text_config: CharConfig::default(),
            feeding: FeedingStyle::Scroll { size: 0, sec: 40. },
            typing_timing: TypingTiming::ByChar { sec: 0.07 },
            writing: WritingStyle::Wipe { sec: 0.07 },
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
