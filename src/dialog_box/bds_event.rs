use bevy::prelude::*;

use crate::read_script::*;
use crate::*;

#[derive(Reflect, Default, Debug)]
pub struct FontSizeChange {
    pub size: f32,
}

#[derive(Reflect, Default, Debug)]
pub struct LoadBds{
    pub path: String,
    pub target_name: String,
}

pub fn change_font_size(
    mut events: EventReader<BdsEvent>,
    mut tb_query: Query<&mut TypeTextConfig, (With<Current>, With<TextBox>)>,
) {
    for event_wrapper in events.iter() {
        if let Some(FontSizeChange { size: s }) = event_wrapper.get_opt::<FontSizeChange>() {
            if let Ok(mut config) = tb_query.get_single_mut() {
                config.text_style.font_size = s;
            }
        }
    }
}

pub fn load_bds(
    mut events: EventReader<BdsEvent>,
    mut db_query: Query<(&DialogBox, &mut LoadedScript)>,
    asset_server: Res<AssetServer>,
){
    for event_wrapper in events.iter() {
        if let Some(LoadBds { path: p, target_name: n }) = event_wrapper.get_opt::<LoadBds>() {
            for (DialogBox{ name: db_name }, mut ls) in &mut db_query {
                if db_name == &n  {
                    let (file, section) = split_path_and_section(&p);
                    ls.bds_handle = asset_server.load(file);
                    ls.target_section = section;
                    ls.order_list = None;
                }
            }
        }
    }
}
