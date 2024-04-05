use crate::dialog_box::window_controller::*;
use crate::read_script::*;
use bevy::prelude::*;

pub struct DebugTextAreaPlugin;

impl Plugin for DebugTextAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, monitor_db_state);
        app.add_systems(Update, monitor_bds_event);
        // app.add_systems(Update, _temporary_update_system);
    }
}

// use crate::dialog_box::input::*;
// pub fn _temporary_update_system(
    // ta_query: Query<&TextArea ,Changed<Selected>>,
// ) {
    // for ta in &ta_query {
        // info!("☛ {:?}", ta.name);
    // }
// }

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
