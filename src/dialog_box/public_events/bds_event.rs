use bevy::prelude::*;

use crate::read_script::*;
use crate::*;

// Reflect登録必須

// Simple String Event的なのを追加したい

#[derive(Reflect, Default, Debug)]
pub struct LoadBds {
    pub path: String,
    pub target_name: String,
}

pub fn load_bds(
    mut events: EventReader<BdsEvent>,
    mut db_query: Query<(&DialogBox, &mut LoadedScript)>,
    asset_server: Res<AssetServer>,
) {
    for event_wrapper in events.read() {
        if let Some(LoadBds {
            path: p,
            target_name: n,
        }) = event_wrapper.get_opt::<LoadBds>()
        {
            for (DialogBox { name: db_name }, mut ls) in &mut db_query {
                if db_name == &n {
                    let (file, section) = split_path_and_section(&p);
                    ls.bds_handle = asset_server.load(file);
                    ls.target_section = section;
                    ls.order_list = None;
                }
            }
        }
    }
}

#[derive(Reflect, Default, Debug)]
pub struct ChangeFontSize {
    pub size: f32,
}

pub fn change_font_size(
    mut events: EventReader<BdsEvent>,
    mut ta_query: Query<&mut TypeTextConfig, (With<Current>, With<TextArea>)>,
) {
    for event_wrapper in events.read() {
        if let Some(ChangeFontSize { size: s }) = event_wrapper.get_opt::<ChangeFontSize>() {
            if let Ok(mut config) = ta_query.get_single_mut() {
                config.text_style.font_size = s;
            }
        }
    }
}

#[derive(Reflect, Default, Debug)]
pub struct ChangeCurrentTextArea {
    target_dialog_box_name: String,
    next_current_text_area_name: String,
}

pub fn change_current_text_area(
    mut commands: Commands,
    db_query: Query<&DialogBox>,
    ta_query: Query<(Entity, &TextArea, &Parent)>,
    mut events: EventReader<BdsEvent>,
){
    for event_wrapper in events.read() {
        if let Some(ChangeCurrentTextArea {
            target_dialog_box_name: db_name ,
            next_current_text_area_name: ta_name ,
        }) = event_wrapper.get_opt::<ChangeCurrentTextArea>() {
            for (entity, text_area, parent) in &ta_query {
                if let Ok(parent_db) = db_query.get(parent.get()) {
                    if parent_db.name == db_name {
                        if ta_name == text_area.name {
                            commands.entity(entity).insert(Current);
                        } else {
                            commands.entity(entity).remove::<Current>();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Reflect, Default, Debug)]
pub struct ChangeCurrentDialogBox {
    next_current_dialog_box_name: String,
}

pub fn change_current_dialog_box(
    mut commands: Commands,
    db_query: Query<(Entity, &DialogBox)>,
    mut events: EventReader<BdsEvent>,
){
    for event_wrapper in events.read() {
        if let Some(ChangeCurrentDialogBox {
            next_current_dialog_box_name: name ,
        }) = event_wrapper.get_opt::<ChangeCurrentDialogBox>() {
            for (entity, dialog_box) in &db_query {
                if name == dialog_box.name {
                    commands.entity(entity).insert(Current);
                } else {
                    commands.entity(entity).remove::<Current>();
                }
            }
        }
    }
}
