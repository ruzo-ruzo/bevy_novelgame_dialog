use crate::dialog_box::bds_event::*;
use crate::dialog_box::input::*;
use crate::dialog_box::window_controller::*;
use bevy::prelude::*;

pub struct DebugTextAreaPlugin;

impl Plugin for DebugTextAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, monitor_db_state);
        app.add_systems(Update, monitor_bds_event);
        app.add_systems(Update, too_many_selected);
        app.add_systems(Update, too_many_current_dialog_box);
    }
}

pub fn too_many_selected(ta_query: Query<&TextArea, (With<Selected>, Without<Pending>)>) {
    let selected_num = ta_query.iter().len();
    if selected_num > 1 {
        error!(
            "there are {:?} non pending selected text areas.",
            selected_num
        );
    }
}

pub fn too_many_current_dialog_box(ta_query: Query<&DialogBox, With<Current>>) {
    let current_num = ta_query.iter().len();
    if current_num > 1 {
        error!("there are {:?} current dialog boxes.", current_num);
    }
}

pub fn monitor_db_state(dbs_query: Query<(&DialogBox, &DialogBoxPhase), Changed<DialogBoxPhase>>) {
    for (db, dbs) in &dbs_query {
        info!("Dialog Box \"{}\"'s State: {dbs:?}", db.name);
    }
}

pub fn monitor_bds_event(mut events: EventReader<BdsEvent>) {
    for event_wrapper in events.read() {
        info!("Throw Event: {:?}", &event_wrapper.value);
    }
}
