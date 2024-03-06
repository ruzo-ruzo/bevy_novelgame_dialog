use super::*;
use crate::read_script::*;

#[derive(Event, Default, Debug, Reflect)]
pub struct SetupChoice {
    pub target_list: Vec<(String, String)>,
}

#[derive(Event, Default, Debug)]
pub struct ChoosenEvent {
    pub choosen_event: String,
}

#[allow(clippy::type_complexity)]
pub fn setup_choice(
    mut commands: Commands,
    mut db_query: Query<
        (Entity, &ChoiceBoxConfig, &mut DialogBoxState),
        (With<Current>, With<DialogBox>),
    >,
    mut events: EventReader<BdsEvent>,
    mut wrapper: EventWriter<ChoosenEvent>,
) {
    for event_wrapper in events.read() {
        if let Some(SetupChoice { target_list: tl }) = event_wrapper.get_opt::<SetupChoice>() {
            if let Ok((db_entity, cbc, mut dbs)) = db_query.get_single_mut() {
                let cs = ChoiceBoxState {
                    main_dialog_box: db_entity,
                    button_entities: cbc.button_entities.clone(),
                    open_dialog_event: OpenDialogEvent {
                        dialog_box_entity: cbc.background_entities,
                        writing: WritingStyle::Put,
                        typing_timing: TypingTiming::ByPage,
                        dialog_box_name: cbc.dialog_box_name.clone(),
                        main_alignment: cbc.main_alignment,
                        popup: cbc.popup,
                        ..default()
                    },
                    target_list: tl.clone(),
                    button_box_origin: cbc.button_box_origin,
                    button_box_size: cbc.button_box_size,
                };
                commands.spawn((cs, Current));
                commands.entity(db_entity).remove::<Current>();
                *dbs = DialogBoxState::Pending;
            }
            // wrapper.send(ChoosenEvent {
            // choosen_event: tl[0].1.clone(),
            // });
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
