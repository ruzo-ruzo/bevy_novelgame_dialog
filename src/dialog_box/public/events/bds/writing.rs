use super::*;

// 倍数版欲しいかも
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

//---------

#[derive(Reflect, Default, Debug)]
pub struct InputForFeeding {
    pub dialog_box_name: String,
    pub text_area_name: String,
}

//---------

#[derive(Reflect, Default, Debug)]
pub struct ForceFeedingCurrentBox;

pub fn force_feeding_current_box(
    mut commands: Commands,
    mut dialog_box_query: Query<(Entity, &DialogBox, &mut DialogBoxPhase), With<Current>>,
    text_area_query: Query<(&TextArea, &Parent)>,
    mut events: EventReader<BdsEvent>,
) {
    for event_wrapper in events.read() {
        if event_wrapper.get_opt::<ForceFeedingCurrentBox>().is_some() {
            for (db_entity, db, mut phase) in &mut dialog_box_query {
                for (ta, ta_parent) in &text_area_query {
                    if ta_parent.get() == db_entity {
                        let iff = InputForFeeding {
                            dialog_box_name: db.name.clone(),
                            text_area_name: ta.name.clone(),
                        };
                        commands.add(|w: &mut World| {
                            w.send_event(BdsEvent {
                                value: Box::new(iff),
                            });
                        });
                        *phase = DialogBoxPhase::WaitingAction;
                    }
                }
            }
        }
    }
}
