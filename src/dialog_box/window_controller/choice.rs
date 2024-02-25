use super::*;
use crate::read_script::*;

#[derive(Default, Debug)]
pub struct ChoosingTarget {
    pub choosen_event: String,
}

#[derive(Event, Default, Debug, Reflect)]
pub struct SetupChoice {
    pub target_list: Vec<(String, String)>,
}

#[derive(Event, Default, Debug)]
pub struct ChoosenEvent {
    pub choosen_event: String,
}

pub fn setup_choice(mut events: EventReader<BdsEvent>, mut wrapper: EventWriter<ChoosenEvent>) {
    for event_wrapper in events.read() {
        if let Some(SetupChoice { target_list: tl }) = event_wrapper.get_opt::<SetupChoice>() {
            wrapper.send(ChoosenEvent {
                choosen_event: tl[0].1.clone(),
            });
        }
    }
}

pub fn closing_choice_phase(
    // mut db_query: Query<&DialogBoxState, (With<Current>, With<DialogBox>)>,
    mut events: EventReader<ChoosenEvent>,
    mut wrapper: EventWriter<BdsEvent>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    for ChoosenEvent { choosen_event: ce } in events.read() {
        if let Ok(next) = read_ron(&app_type_registry, ce) {
            wrapper.send(BdsEvent { value: next });
        }
    }
}
