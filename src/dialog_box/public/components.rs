use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct WaitingIcon {
    pub target_window_name: String,
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
pub enum PopupType {
    Scale { sec: f32 },
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum WaitBrakerStyle {
    Auto { wait_sec: f32 },
    Input { is_icon_moving_to_last: bool },
}
