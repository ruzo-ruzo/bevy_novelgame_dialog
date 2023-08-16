use crate::dialog_box::window_controller::*;
use crate::read_script::*;
use bevy::prelude::*;

pub struct DebugTextBoxPlugin;

impl Plugin for DebugTextBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, monitor_db_state);
        app.add_systems(Update, monitor_bds_event);
    }
}

pub fn monitor_db_state(
    dbs_query: Query<&DialogBoxState, Changed<DialogBoxState>>
){
    for dbs in &dbs_query {
        info!("Dialog Box State: {dbs:?}");
    }
}

pub fn monitor_bds_event(
    mut events: EventReader<BdsEvent>,
){
    for event_wrapper in events.iter() {
        info!("Throw Event: {:?}", &event_wrapper.value);
    }
}
