pub mod writing;
pub use writing::*;

use crate::dialog_box::public::components::*;
use crate::dialog_box::public::configs::*;
use crate::dialog_box::window_controller::*;
use crate::read_script::*;
use bevy::prelude::*;

// Reflect登録必須。逆にEventは基本要らない
#[derive(Event)]
pub struct BdsEvent {
    pub value: Box<dyn Reflect>,
}

impl BdsEvent {
    pub fn base<T: Default + Reflect>(&self) -> T {
        let mut my_data = <T>::default();
        my_data.apply(&*self.value);
        my_data
    }

    pub fn get<T: Default + Reflect + TypePath>(&self) -> Option<T> {
        if self.value.represents::<T>() {
            Some(self.base::<T>())
        } else {
            None
        }
    }
}

//-----

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
        }) = event_wrapper.get::<LoadBds>()
        {
            for (DialogBox { name: db_name }, mut ls) in &mut db_query {
                if db_name == &n {
                    let (file, section) = split_path_and_section(&p);
                    ls.bds_handle_opt = Some(asset_server.load(file));
                    ls.target_section = section;
                    ls.order_list = None;
                }
            }
        }
    }
}

//-----

#[derive(Reflect, Default, Debug)]
pub struct SinkDownWindow {
    pub sink_type: SinkDownType,
}

//-----

#[derive(Reflect, Default, Debug, PartialEq)]
pub struct SimpleWait;

#[derive(Reflect, Default, Debug)]
pub struct SimpleStringSignal {
    pub signal: String,
}

#[derive(Event, Default, Debug)]
pub struct BdsSignal {
    pub signal: String,
}

pub fn send_bds_signal(
    mut bds_events: EventReader<BdsEvent>,
    mut signal_events: EventWriter<BdsSignal>,
) {
    for event_wrapper in bds_events.read() {
        if let Some(SimpleStringSignal {
            signal: base_signal,
        }) = event_wrapper.get::<SimpleStringSignal>()
        {
            signal_events.send(BdsSignal {
                signal: base_signal.clone(),
            });
        }
    }
}

//-----

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
) {
    for event_wrapper in events.read() {
        if let Some(ChangeCurrentTextArea {
            target_dialog_box_name: db_name,
            next_current_text_area_name: ta_name,
        }) = event_wrapper.get::<ChangeCurrentTextArea>()
        {
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
pub struct ChangeCurrentTextAreaInCurrentBox {
    next_current_text_area_name: String,
}

pub fn change_current_text_area_in_current_box(
    mut commands: Commands,
    db_query: Query<Entity, (With<DialogBox>, With<Current>)>,
    ta_query: Query<(Entity, &TextArea, &Parent)>,
    mut events: EventReader<BdsEvent>,
) {
    for event_wrapper in events.read() {
        if let Some(ChangeCurrentTextAreaInCurrentBox {
            next_current_text_area_name: ta_name,
        }) = event_wrapper.get::<ChangeCurrentTextAreaInCurrentBox>()
        {
            for (entity, text_area, parent) in &ta_query {
                if db_query.get_single().ok() == Some(parent.get()) {
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

//-----

#[derive(Reflect, Default, Debug)]
pub struct ChangeCurrentDialogBox {
    next_current_dialog_box_name: String,
}

pub fn change_current_dialog_box(
    mut commands: Commands,
    db_query: Query<(Entity, &DialogBox)>,
    mut events: EventReader<BdsEvent>,
) {
    for event_wrapper in events.read() {
        if let Some(ChangeCurrentDialogBox {
            next_current_dialog_box_name: name,
        }) = event_wrapper.get::<ChangeCurrentDialogBox>()
        {
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
