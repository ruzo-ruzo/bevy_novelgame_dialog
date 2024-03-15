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
pub fn open_choice_box(
    mut commands: Commands,
    mut db_query: Query<
        (Entity, &ChoiceBoxConfig, &mut DialogBoxPhase),
        (With<Current>, With<DialogBox>),
    >,
    mut vis_query: Query<&mut Visibility>,
    cc_query: Query<Entity, (With<ChoiceBoxState>, With<Current>)>,
    mut sp_query: Query<&mut Sprite>,
    mut tf_query: Query<&mut Transform>,
    mut events: EventReader<BdsEvent>,
    // mut wrapper: EventWriter<ChoosenEvent>,
    setup_config: Res<SetupConfig>,
    mut ow_event: EventWriter<OpenDialogEvent>,
) {
    for event_wrapper in events.read() {
        if let Some(SetupChoice { target_list: tl }) = event_wrapper.get_opt::<SetupChoice>() {
            if let Ok((db_entity, cbc, mut dbs)) = db_query.get_single_mut() {
                let background_entity = if let Some(entity) = cbc.background_entity {
                    entity
                } else {
                    commands.spawn(SpriteBundle::default()).id()
                };
                let ta_names = cbc
                    .button_text_areas
                    .iter()
                    .map(|c| c.area_name.clone())
                    .collect::<Vec<_>>();
                let cs = ChoiceBoxState {
                    main_dialog_box: db_entity,
                    button_entities: cbc.button_entities.clone(),
                    choice_box_entity: None,
                    target_list: tl.clone(),
                    select_vector: cbc.select_vector,
                };
                cc_query.iter().for_each(|e| {
                    commands.entity(e).remove::<Current>();
                });
                commands
                    .entity(background_entity)
                    .push_children(&cbc.button_entities);
                commands.spawn((cs, Current));
                if let Ok(mut vis) = vis_query.get_mut(background_entity) {
                    *vis = Visibility::Hidden;
                }
                let size = tl.len() as f32;
                let x_expand = cbc.background_scaling_per_button.x*size;
                let y_expand = cbc.background_scaling_per_button.y*size;
                let (x_dir, y_dir) = get_slide_direction(cbc.background_scaling_anchor);
                if let Ok(mut sp) = sp_query.get_mut(background_entity) {
                    sp.custom_size = sp.custom_size.map(|Vec2 { x, y }| {
                            Vec2::new( x + x_expand, y + y_expand )
                    });
                }
                for entity in &cbc.button_entities {
                    if let Ok(mut vis) = vis_query.get_mut(*entity) {
                        let index = cbc.button_entities.iter().position(|x| x == entity);
                        if index.unwrap() >= tl.len() {
                            *vis = Visibility::Hidden;
                        } else {
                            *vis = Visibility::Inherited;
                        }
                    }
                    commands
                        .entity(*entity)
                        .insert(RenderLayers::layer(setup_config.render_layer));
                    if let Ok(mut tf) = tf_query.get_mut(*entity) {
                        tf.translation.x += x_dir*x_expand/2.0;
                        tf.translation.y +=y_dir*y_expand/2.0;
                    }
                }
                let slided_text_area_configs = cbc.button_text_areas.iter().map(|base|{
                    TextAreaConfig {
                        area_origin: Vec2::new(
                            base.area_origin.x + x_dir*x_expand/2.0,
                            base.area_origin.y + y_dir*y_expand/2.0,
                        ),
                        .. base.clone()
                    }}).collect::<Vec<_>>();
                let opening_event = OpenDialogEvent {
                    dialog_box_entity: Some(background_entity),
                    dialog_box_name: cbc.dialog_box_name.clone(),
                    raw_orders: make_choice_order(&tl, &cbc.dialog_box_name, &ta_names),
                    popup: cbc.popup,
                    text_area_configs: slided_text_area_configs,
                    ..default()
                };
                // ow_event.send(opening_event);
                // *dbs = DialogBoxPhase::Pending;
            }
            wrapper.send(ChoosenEvent {
                choosen_event: tl[0].1.clone(),
            });
        }
    }
}

fn get_slide_direction(anchor: Anchor)  -> (f32, f32) {
    let mut x_direction = 0.0;
    let mut y_direction = 0.0;
    if anchor == Anchor::TopLeft || anchor == Anchor::TopCenter || anchor == Anchor::TopRight {
            y_direction = 1.0;
    }
    else if anchor == 
        Anchor::BottomLeft || anchor == Anchor::BottomCenter || anchor == Anchor::BottomRight {
            y_direction = -1.0;
    }
    if anchor ==
        Anchor::TopLeft || anchor == Anchor::CenterLeft || anchor == Anchor::BottomLeft {
            x_direction = -1.0;
    }
    else if anchor == 
        Anchor::TopRight || anchor == Anchor::CenterRight || anchor == Anchor::BottomRight {
            x_direction = 1.0;
    }
    (x_direction, y_direction)
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
        script.push_str(&format!(
            "{header}{dialog_box_name}{midpoint}{name}{footer}"
        ));
        script.push_str(text);
    }
    Some(parse_script(&script, "", ""))
}

pub fn setup_choice(
    mut cb_query: Query<&mut ChoiceBoxState, With<Current>>,
    db_query: Query<(Entity, &DialogBoxPhase),  With<Current>>,
) {
    if let Ok(mut cbs) = cb_query.get_single_mut(){
        if cbs.choice_box_entity.is_none() {
            if let Ok((entity, dbp)) = db_query.get_single() {
                if *dbp != DialogBoxPhase::Pending  { cbs.choice_box_entity = Some(entity) };
            }
        }
    }
}

pub fn close_choice_phase(
    // mut db_query: Query<&DialogBoxPhase, With<DialogBox>>,
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
