mod choice_box;
mod main_box;

use crate::prelude::*;
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use choice_box::ChoiceBoxPlugIn;
use main_box::MainBoxPlugIn;

const ASSETS_PATH: &str = "embedded://bevy_novelgame_dialog/ui_templates/fantasy_style/../assets/";

#[derive(Resource, Default)]
pub struct TemplateSetupConfig {
    pub render_layer: u8,
    pub render_order: isize,
}

pub struct RPGStyleUIPlugin {
    pub layer_num: u8,
    pub render_order: isize,
}

impl Default for RPGStyleUIPlugin {
    fn default() -> Self {
        RPGStyleUIPlugin {
            layer_num: 2,
            render_order: 1,
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
            render_order: self.render_order,
        })
        .add_event::<OpenRPGStyleDialog>()
        .add_systems(Update, open_message);
    }
}

struct EmbeddedAssetPlugin;

impl Plugin for EmbeddedAssetPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "../assets/common/scripts/basic.csv");
        embedded_asset!(app, "../assets/fantasy_style/scripts/basic.csv");
        embedded_asset!(
            app,
            "../assets/fantasy_style/fonts/UnifrakturMaguntia/UnifrakturMaguntia-Regular.ttf"
        );
        embedded_asset!(
            app,
            "../assets/fantasy_style/fonts/赤薔薇/akabara-cinderella.ttf"
        );
        embedded_asset!(app, "../assets/fantasy_style/fonts/网风雅宋/网风雅宋.ttf");
        embedded_asset!(
            app,
            "../assets/fantasy_style/fonts/noto/NotoEmoji-VariableFont_wght.ttf"
        );
    }
}

#[derive(Event)]
pub struct OpenRPGStyleDialog {
    pub script_path: String,
}

fn open_message(
    mut open_message_event: EventReader<OpenRPGStyleDialog>,
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
        .map(|s| ASSETS_PATH.to_owned() + "fantasy_style/fonts/" + s);
        let font_vec = font_path_vec
            .zip([(1.0, 0.0), (1.0, 0.0), (1.3, 0.0), (1.0, 0.0)].iter())
            .map(|(p, (s, k))| FontConfig {
                path: p.clone(),
                kerning: *k,
                size: *s,
            })
            .collect::<Vec<_>>();
        let frame_tac = TextAreaConfig {
            area_name: "Main Area".to_string(),
            font_sets: font_vec.clone(),
            feeding: FeedingStyle::Scroll { size: 0, sec: 0.5 },
            font_color: Color::rgb(0.7, 0.5, 0.3),
            text_base_size: 32.0,
            area_origin: Vec2::new(-520.0, 70.0),
            area_size: Vec2::new(1010.0, 140.0),
            ..default()
        };
        let name_plate_tac = TextAreaConfig {
            area_name: "Name Area".to_string(),
            area_origin: Vec2::new(-480.0, 170.0),
            area_size: Vec2::new(400.0, 80.0),
            font_sets: font_vec.clone(),
            font_color: Color::ANTIQUE_WHITE,
            text_base_size: 32.0,
            feeding: FeedingStyle::Rid,
            writing: WritingStyle::Put,
            typing_timing: TypingTiming::ByPage,
            vertical_alignment: AlignVertical::Center,
            ..default()
        };
        let tac_base = TextAreaConfig {
            font_sets: font_vec.clone(),
            area_size: Vec2::new(360.0, 100.0),
            font_color: Color::NAVY,
            writing: WritingStyle::Put,
            typing_timing: TypingTiming::ByPage,
            horizon_alignment: AlignHorizon::Center,
            vertical_alignment: AlignVertical::Center,
            ..default()
        };
        let tac_list = (0..4)
            .map(|i| TextAreaConfig {
                area_origin: Vec2::new(-180.0, -20.0 - 140.0 * (i as f32)),
                area_name: format!("Button Area {i:02}"),
                ..tac_base.clone()
            })
            .collect::<Vec<_>>();
        ow_event.send(OpenDialog {
            dialog_box_name: "Main Box".to_string(),
            script_path: path.clone(),
            template_path: vec![
                (ASSETS_PATH.to_owned() + "fantasy_style/scripts/basic.csv").to_string(),
                (ASSETS_PATH.to_owned() + "common/scripts/basic.csv").to_string(),
            ],
            text_area_configs: vec![frame_tac, name_plate_tac],
            position: Vec2::new(0., -200.),
            wait_breaker: WaitBrakerStyle::Input {
                is_icon_moving_to_last: true,
                is_all_range_area: true,
            },
            template_open_choice: ChoiceBoxConfig {
                choice_box_name: "Choice Box".to_string(),
                button_text_areas: tac_list,
                background_scaling_per_button: Vec2::new(0.0, 140.0),
                background_scaling_anchor: Anchor::TopCenter,
                ..default()
            },
            ..default()
        });
    }
}
