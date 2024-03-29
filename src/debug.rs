use crate::dialog_box::window_controller::*;
use crate::read_script::*;
use bevy::prelude::*;

pub struct DebugTextAreaPlugin;

impl Plugin for DebugTextAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, monitor_db_state);
        app.add_systems(Update, monitor_bds_event);
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
