pub mod bds_event;

use super::window_controller::*;
use crate::read_script::*;
use bevy::prelude::*;

#[derive(Event)]
pub struct OpenDialogEvent {
    pub dialog_box_name: String,
    pub dialog_box_entity: Option<Entity>,
    pub position: Vec2,
    pub popup: PopupType,
    pub wait_breaker: WaitBrakerStyle,
    pub script_path: String,
    pub template_path: String,
    pub raw_orders: Option<Vec<Order>>,
    pub template_open_choice: ChoiceBoxConfig,
    pub text_area_configs: Vec<TextAreaConfig>,
    pub main_text_area_name: String,
}

impl Default for OpenDialogEvent {
    fn default() -> Self {
        OpenDialogEvent {
            dialog_box_name: "Main Box".to_string(),
            dialog_box_entity: None,
            position: Vec2::new(0., 0.),
            popup: PopupType::Scale { sec: 0.8 },
            wait_breaker: WaitBrakerStyle::Auto { wait_sec: 1.5 },
            script_path: "scripts/message.bds".to_string(),
            template_path: "scripts/template.bdt".to_string(),
            raw_orders: None,
            template_open_choice: ChoiceBoxConfig::default(),
            text_area_configs: vec![TextAreaConfig::default()],
            main_text_area_name: "Main Area".to_string(),
        }
    }
}

#[derive(Event)]
pub struct SelectedEvent(pub Entity);

#[derive(Event)]
pub struct GoSelectedEvent(pub Entity);
