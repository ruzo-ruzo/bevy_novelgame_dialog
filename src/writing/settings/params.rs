use bevy::prelude::*;

// Todo: ページング用とアイコン分けたい
// 移動するかどうかの設定をこっちに持たせる？
#[derive(Component, Debug, Default)]
pub struct WaitingIcon {
    pub target_box_name: String,
    pub wait_for: Vec<WaitTarget>,
}

#[derive(Clone)]
pub struct FontSettings {
    pub path: String,
    pub size_coefficient: f32,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            path: "fonts/FiraMono-Regular.ttf".to_string(),
            size_coefficient: 1.0,
        }
    }
}

#[derive(Component, Debug, PartialEq)]
pub enum WaitTarget {
    Feeding,
    SimpleWaiting,
}

#[derive(Component, Debug, Default)]
pub struct DialogBoxBackground {
    pub writing_name: String,
}

#[derive(Component)]
pub struct ChoiceButton {
    pub target_box_name: String,
    pub sort_number: usize,
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

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub enum WritingStyle {
    Wipe {
        sec: f32,
    },
    #[default]
    Put,
    // Scroll  { size: usize, sec: f32 },
    // Fade,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum FeedingStyle {
    Scroll { size: usize, sec: f32 },
    Rid,
    // Fade,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum PopupType {
    Scale { sec: f32 },
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum WaitBrakerStyle {
    Auto {
        wait_sec: f32,
    },
    Input {
        is_icon_moving_to_last: bool,
        is_all_range_area: bool,
    },
}

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
