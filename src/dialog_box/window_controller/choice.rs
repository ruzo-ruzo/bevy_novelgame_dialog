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
    mut vis_query: Query<&mut Visibility>,
    cc_query: Query<Entity, (With<ChoiceBoxState>, With<Current>)>,
    mut events: EventReader<BdsEvent>,
    mut wrapper: EventWriter<ChoosenEvent>,
    setup_config: Res<SetupConfig>,
) {
    for event_wrapper in events.read() {
        if let Some(SetupChoice { target_list: tl }) = event_wrapper.get_opt::<SetupChoice>() {
            if let Ok((db_entity, cbc, mut dbs)) = db_query.get_single_mut() {
                let background_entity = if let Some(entity) = cbc.background_entity {
                        entity
                    } else {
                        commands.spawn(SpriteBundle::default()).id()
                    };
                let cs = ChoiceBoxState {
                    main_dialog_box: db_entity,
                    button_entities: cbc.button_entities.clone(),
                    open_dialog_event: OpenDialogEvent {
                        dialog_box_entity: Some(background_entity),
                        dialog_box_name: cbc.dialog_box_name.clone(),
                        raw_orders: make_choice_order(&tl),
                        popup: cbc.popup,
                        text_area_configs: vec![TextAreaConfig {
                            area_name: "Button Area 01".to_string(),
                            writing: WritingStyle::Put,
                            typing_timing: TypingTiming::ByPage,
                            main_alignment: cbc.main_alignment,
                            text_pos_z: cbc.text_pos_z,
                            ..default()
                        }],
                        ..default()
                    },
                    target_list: tl.clone(),
                    select_vector: cbc.select_vector,
                    button_box_origin: cbc.button_box_origin,
                    button_box_size: cbc.button_box_size,
                };
                cc_query.iter().for_each(|e| {commands.entity(e).remove::<Current>();});
                commands.spawn((cs, Current));
                commands.entity(background_entity).push_children(&cbc.button_entities);
                for entity in &cbc.button_entities {
                    if let Ok(mut vis) = vis_query.get_mut(*entity) {
                        *vis = Visibility::Hidden;
                    }
                    commands.entity(*entity).insert(RenderLayers::layer(setup_config.render_layer));
                }
                // *dbs = DialogBoxState::Pending;
            }
            wrapper.send(ChoosenEvent {
                choosen_event: tl[0].1.clone(),
            });
        }
    }
}

#[derive(Event, Default, Debug, Reflect)]
pub struct SetupChoiceAreas;

fn make_choice_order(target_list: &[(String, String)]) -> Option<Vec<Order>> {
    Some(parse_script("", "", ""))
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
