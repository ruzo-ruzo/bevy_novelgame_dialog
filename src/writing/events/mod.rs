pub mod bds;
pub use bds::*;

use crate::writing::settings::configs::*;
use crate::writing::settings::params::*;
use bevy::prelude::*;

#[derive(Event)]
pub struct OpenDialog {
    pub writing_name: String,
    pub position: Vec2,
    pub popup: PopupType,
    pub wait_breaker: WaitBrakerStyle,
    pub script_path: String,
    pub template_path: Vec<String>,
    pub raw_orders: Option<Vec<Order>>,
    pub template_open_choice: ChoiceBoxConfig,
    pub text_area_configs: Vec<TextAreaConfig>,
    pub main_text_area_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    Type { character: char },
    CarriageReturn,
    PageFeed,
    ThroghEvent { ron: String },
}

impl Default for OpenDialog {
    fn default() -> Self {
        OpenDialog {
            writing_name: "Main Box".to_string(),
            position: Vec2::new(0., 0.),
            popup: PopupType::Scale { sec: 0.8 },
            wait_breaker: WaitBrakerStyle::Auto { wait_sec: 1.5 },
            script_path: "scripts/message.bds".to_string(),
            template_path: vec!["scripts/template.bdt".to_string()],
            raw_orders: None,
            template_open_choice: ChoiceBoxConfig::default(),
            text_area_configs: vec![TextAreaConfig::default()],
            main_text_area_name: "Main Area".to_string(),
        }
    }
}

#[derive(Event, Debug)]
pub struct ButtonIsSelected {
    pub writing_name: String,
    pub text_area_name: String,
    pub select_vector: SelectVector,
    pub select_number: usize,
}

#[derive(Event)]
pub struct ButtonIsPushed {
    pub writing_name: String,
    pub text_area_name: String,
}

#[derive(Event)]
pub struct FinisClosingBox {
    pub writing_name: String,
}
