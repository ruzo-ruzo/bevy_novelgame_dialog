use super::*;

#[derive(Reflect, Default, Debug)]
pub struct ChoosingTarget {
    pub choosen_event: String,
}

#[derive(Event, Default, Debug)]
pub struct SetupChoice {
    pub target_list: Vec<(String, String)>,
}

#[derive(Event, Default, Debug)]
pub struct ChoosenEvent {
    pub choosen_event: String,
}

pub fn setup_choice(
) {
    
}

pub fn closing_choice_phase(
    mut db_query: Query<&DialogBoxState, (With<Current>, With<DialogBox>)>,
    mut events: EventReader<ChoosenEvent>,
) {
    
}
