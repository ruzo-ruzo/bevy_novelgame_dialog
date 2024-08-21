mod choice_box;
mod main_box;

use crate::prelude::*;
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::color::palettes::css as CssColor;
use choice_box::ChoiceBoxPlugIn;
use main_box::MainBoxPlugIn;
use std::collections::HashMap;

const COMMON_PATH: &str = "embedded://bevy_novelgame_dialog/ui_templates/fantasy_style/../assets/";
const ASSETS_PATH: &str = "embedded://bevy_novelgame_dialog/ui_templates/fantasy_style/assets/";

#[derive(Resource, Default)]
struct TemplateSetupConfig {
    render_layer: u8,
    box_size: Vec2,
    box_pos: Vec2,
    choice_pos: Vec2,
    button_size: Vec2,
    max_button_index: usize,
    font_size: f32,
}

/// `RPGStyleUIPlugin` is a plugin for creating RPG-style text box.
pub struct RPGStyleUIPlugin {
    /// The layer number of the UI. Specifies the order in which layers overlap.
    pub layer_num: u8,
    /// Specifies the render order. Higher values are rendered in front.
    pub render_order: isize,
    /// Specifies the size of the text box.
    pub box_size: Vec2,
    /// Specifies the position of the text box.
    pub box_pos: Vec2,
    /// Specifies the position of the choice buttons box.
    pub choice_pos: Vec2,
    /// Specifies the size of the choice buttons.
    pub button_size: Vec2,
    /// Specifies the maximum number of choice buttons.
    pub max_button_index: usize,
    /// Specifies the font size.
    pub font_size: f32,
}

impl Default for RPGStyleUIPlugin {
    fn default() -> Self {
        RPGStyleUIPlugin {
            layer_num: 2,
            render_order: 1,
            box_size: Vec2::new(1200.0, 300.0),
            box_pos: Vec2::new(0.0, -200.0),
            choice_pos: Vec2::new(0.0, -200.0),
            button_size: Vec2::new(400.0, 100.0),
            max_button_index: 3,
            font_size: 32.0,
        }
    }
}

impl Plugin for RPGStyleUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DialogBoxPlugin {
                layer_num: self.layer_num,
                render_order: self.render_order,
            },
            EmbeddedAssetPlugin,
            MainBoxPlugIn,
            ChoiceBoxPlugIn,
        ))
        .insert_resource(TemplateSetupConfig {
            render_layer: self.layer_num,
            box_size: self.box_size,
            box_pos: self.box_pos,
            choice_pos: self.choice_pos,
            button_size: self.button_size,
            max_button_index: self.max_button_index,
            font_size: self.font_size,
        })
        .add_event::<OpenRPGStyleDialog>()
        .add_systems(Update, open_message);
    }
}

struct EmbeddedAssetPlugin;

impl Plugin for EmbeddedAssetPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "../assets/scripts/basic.csv");
        embedded_asset!(app, "assets/scripts/custom.csv");
        embedded_asset!(
            app,
            "assets/fonts/UnifrakturMaguntia/UnifrakturMaguntia-Regular.ttf"
        );
        embedded_asset!(app, "assets/fonts/赤薔薇/akabara-cinderella.ttf");
        embedded_asset!(app, "assets/fonts/网风雅宋/网风雅宋.ttf");
        embedded_asset!(app, "assets/fonts/noto/NotoEmoji-VariableFont_wght.ttf");
    }
}

#[derive(Event)]
pub struct OpenRPGStyleDialog {
    pub script_path: String,
}

fn open_message(
    mut open_message_event: EventReader<OpenRPGStyleDialog>,
    config: Res<TemplateSetupConfig>,
    mut ow_event: EventWriter<OpenDialog>,
) {
    for OpenRPGStyleDialog { script_path: path } in open_message_event.read() {
        let font_path_vec = [
            "UnifrakturMaguntia/UnifrakturMaguntia-Regular.ttf",
            "赤薔薇/akabara-cinderella.ttf",
            "网风雅宋/网风雅宋.ttf",
            "noto/NotoEmoji-VariableFont_wght.ttf",
        ]
        .iter()
        .map(|s| ASSETS_PATH.to_owned() + "fonts/" + s)
        .collect::<Vec<_>>();
        let text_conf = CharConfig {
            font_paths: font_path_vec,
            kerning_by_regulars: HashMap::from([(" ".to_string(), -0.7)]),
            size_by_regulars: HashMap::from([("[[:alpha:]]".to_string(), 1.2)]),
            text_base_size: config.font_size,
            font_color: Color::srgb(0.7, 0.5, 0.3),
        };
        let text_area_x = -config.box_size.x / 2.0 + config.box_pos.x + 80.0;
        let text_area_y = config.box_size.y / 2.0 + config.box_pos.y + 120.0;
        let frame_tac = TextAreaConfig {
            area_name: "Main Area".to_string(),
            text_config: text_conf.clone(),
            feeding: FeedingStyle::Scroll { size: 0, sec: 0.5 },
            area_origin: Vec2::new(text_area_x, text_area_y),
            area_size: Vec2::new(config.box_size.x - 90.0, config.box_size.y - 160.0),
            ..default()
        };
        let name_area_x = -(config.box_size.x / 2.0) + 100.0;
        let name_area_y = config.box_size.y / 2.0 + 18.0;
        let name_plate_tac = TextAreaConfig {
            area_name: "Name Area".to_string(),
            area_origin: Vec2::new(name_area_x, name_area_y),
            area_size: Vec2::new(400.0, 80.0),
            text_config: CharConfig {
                font_color: CssColor::ANTIQUE_WHITE.into(),
                ..text_conf.clone()
            },
            feeding: FeedingStyle::Rid,
            writing: WritingStyle::Put,
            typing_timing: TypingTiming::ByPage,
            vertical_alignment: AlignVertical::Center,
            ..default()
        };
        let tac_base = TextAreaConfig {
            text_config: CharConfig {
                font_color: CssColor::NAVY.into(),
                ..text_conf.clone()
            },
            area_size: Vec2::new(config.button_size.x - 40.0, config.button_size.y),
            writing: WritingStyle::Put,
            typing_timing: TypingTiming::ByPage,
            horizon_alignment: AlignHorizon::Center,
            vertical_alignment: AlignVertical::Center,
            ..default()
        };
        let tac_list = (0..config.max_button_index)
            .map(|i| {
                let button_x = -config.button_size.x / 2.0 + 20.0;
                let button_y = -20.0 - (config.button_size.y + 40.0) * (i as f32);
                TextAreaConfig {
                    area_origin: Vec2::new(button_x, button_y),
                    area_name: format!("Button Area {i:02}"),
                    ..tac_base.clone()
                }
            })
            .collect::<Vec<_>>();
        ow_event.send(OpenDialog {
            writing_name: "Main Box".to_string(),
            script_path: path.clone(),
            template_path: vec![
                (ASSETS_PATH.to_owned() + "scripts/custom.csv").to_string(),
                (COMMON_PATH.to_owned() + "scripts/basic.csv").to_string(),
            ],
            text_area_configs: vec![frame_tac, name_plate_tac],
            position: config.choice_pos,
            wait_breaker: WaitBrakerStyle::Input {
                is_icon_moving_to_last: true,
                is_all_range_area: true,
            },
            template_open_choice: ChoiceBoxConfig {
                choice_box_name: "Choice Box".to_string(),
                button_text_areas: tac_list,
                background_scaling_per_button: Vec2::new(0.0, config.button_size.y + 40.0),
                background_scaling_anchor: Anchor::TopCenter,
                ..default()
            },
            ..default()
        });
    }
}
