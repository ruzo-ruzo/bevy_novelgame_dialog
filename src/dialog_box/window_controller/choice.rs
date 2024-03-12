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
        (Entity, &ChoiceBoxConfig, &mut DialogBoxPhase),
        (With<Current>, With<DialogBox>),
    >,
    mut vis_query: Query<&mut Visibility>,
    cc_query: Query<Entity, (With<ChoiceBoxState>, With<Current>)>,
    mut events: EventReader<BdsEvent>,
    mut wrapper: EventWriter<ChoosenEvent>,
    setup_config: Res<SetupConfig>,
    mut ow_event: EventWriter<OpenDialogEvent>,
) {
    for event_wrapper in events.read() {
        if let Some(SetupChoice { target_list: tl }) = event_wrapper.get_opt::<SetupChoice>() {
            // if let Ok((db_entity, cbc, mut dbs)) = db_query.get_single_mut() {
                // let background_entity = if let Some(entity) = cbc.background_entity {
                        // entity
                    // } else {
                        // commands.spawn(SpriteBundle::default()).id()
                    // };
                // let ta_names = cbc.button_text_areas.iter().map(|c| c.area_name.clone()).collect::<Vec<_>>();
                // let opening_event = OpenDialogEvent {
                    // dialog_box_entity: Some(background_entity),
                    // dialog_box_name: cbc.dialog_box_name.clone(),
                    // raw_orders: make_choice_order(&tl, &cbc.dialog_box_name, &ta_names),
                    // popup: cbc.popup,
                    // text_area_configs: cbc.button_text_areas.clone(),
                    // ..default()
                // };
                // let cs = ChoiceBoxState {
                    // main_dialog_box: db_entity,
                    // button_entities: cbc.button_entities.clone(),
                    // open_dialog_event: opening_event.clone(),
                    // target_list: tl.clone(),
                    // select_vector: cbc.select_vector,
                // };
                // cc_query.iter().for_each(|e| {commands.entity(e).remove::<Current>();});
                // commands.entity(background_entity).push_children(&cbc.button_entities);
                // commands.spawn((cs, Current));
                // for entity in &cbc.button_entities {
                    // if let Ok(mut vis) = vis_query.get_mut(*entity) {
                        // *vis = Visibility::Inherited;
                    // }
                    // commands.entity(*entity).insert(RenderLayers::layer(setup_config.render_layer));
                // }
                // *dbs = DialogBoxPhase::Pending;
                // ow_event.send(opening_event);
            // }
            wrapper.send(ChoosenEvent {
                choosen_event: tl[0].1.clone(),
            });
        }
    }
}

fn make_choice_order(
    target_list: &[(String, String)],
    dialog_box_name: &str,
    text_area_names: &[String],
) -> Option<Vec<Order>> {
    let header = r#"<script>{
        "bevy_novelgame_dialog::dialog_box::public_events::bds_event::ChangeCurrentTextArea": 
        (target_dialog_box_name: ""#;
    let midpoint = r#"",  next_current_text_area_name: ""#;
    let footer = r#"",),}</script>"#;
    let mut script = String::new();
    for ((text, _), name) in target_list.iter().zip(text_area_names.iter()) {
        script.push_str(&format!("{header}{dialog_box_name}{midpoint}{name}{footer}"));
        script.push_str(text);
    }
    Some(parse_script(&script, "", ""))
}

pub fn closing_choice_phase(
    // mut db_query: Query<&DialogBoxPhase, (With<Current>, With<DialogBox>)>,
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
