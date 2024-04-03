use super::*;
use crate::dialog_box::*;
use bevy::render::view::RenderLayers;

#[derive(Event, Default, Debug, Reflect)]
pub struct SetupChoice {
    target_list: Vec<(String, String)>,
}

#[derive(Reflect, Default, Debug)]
pub struct ChoosenEvent {
    pub choosen_event: String,
    pub choice_box_state_entity: Option<Entity>,
}

#[allow(clippy::type_complexity)]
pub fn open_choice_box(
    mut commands: Commands,
    mut db_query: Query<
        (Entity, &ChoiceBoxConfig, &mut DialogBoxPhase, &Children),
        (With<Current>, With<DialogBox>),
    >,
    mut vis_query: Query<&mut Visibility>,
    cc_query: Query<Entity, (With<ChoiceBoxState>, With<Current>)>,
    mut sp_query: Query<&mut Sprite>,
    mut tf_query: Query<&mut Transform>,
    mut events: EventReader<BdsEvent>,
    setup_config: Res<SetupConfig>,
    mut ow_event: EventWriter<OpenDialogEvent>,
) {
    for event_wrapper in events.read() {
        if let Some(SetupChoice { target_list: tl }) = event_wrapper.get_opt::<SetupChoice>() {
            if let Ok((db_entity, cbc, mut dbs, children)) = db_query.get_single_mut() {
                let background_entity = if let Some(entity) = cbc.background_entity {
                    entity
                } else {
                    commands.spawn(SpriteBundle::default()).id()
                };
                let culled_tas = &cbc.button_text_areas[0..tl.len()];
                let ta_names = culled_tas
                    .iter()
                    .map(|c| c.area_name.clone())
                    .collect::<Vec<_>>();
                let cs = ChoiceBoxState {
                    main_dialog_box: db_entity,
                    button_entities: cbc.button_entities.clone(),
                    choice_box_entity: None,
                    target_list: tl.clone(),
                    select_vector: cbc.select_vector,
                    sinkdown: cbc.sinkdown,
                    text_area_names: ta_names.clone(),
                    background_scaling_per_button: cbc.background_scaling_per_button,
                    background_scaling_anchor: cbc.background_scaling_anchor,
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
                let x_expand = cbc.background_scaling_per_button.x * size;
                let y_expand = cbc.background_scaling_per_button.y * size;
                let (x_dir, y_dir) = get_slide_direction(cbc.background_scaling_anchor);
                if let Ok(mut sp) = sp_query.get_mut(background_entity) {
                    sp.custom_size = sp
                        .custom_size
                        .map(|Vec2 { x, y }| Vec2::new(x + x_expand, y + y_expand));
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
                        tf.translation.x += x_dir * x_expand / 2.0;
                        tf.translation.y += y_dir * y_expand / 2.0;
                    }
                }
                let slided_text_area_configs = culled_tas
                    .iter()
                    .map(|base| TextAreaConfig {
                        area_origin: Vec2::new(
                            base.area_origin.x + x_dir * x_expand / 2.0,
                            base.area_origin.y + y_dir * y_expand / 2.0,
                        ),
                        ..base.clone()
                    })
                    .collect::<Vec<_>>();
                let opening_event = OpenDialogEvent {
                    dialog_box_entity: Some(background_entity),
                    dialog_box_name: cbc.dialog_box_name.clone(),
                    raw_orders: make_choice_order(&tl, &cbc.dialog_box_name, &ta_names),
                    popup: cbc.popup,
                    text_area_configs: slided_text_area_configs,
                    ..default()
                };
                ow_event.send(opening_event);
                *dbs = DialogBoxPhase::Fixed;
                for childe in children {
                    commands.entity(*childe).insert(Pending);
                }
            }
        }
    }
}

fn get_slide_direction(anchor: Anchor) -> (f32, f32) {
    let mut x_direction = 0.0;
    let mut y_direction = 0.0;
    if anchor == Anchor::TopLeft || anchor == Anchor::TopCenter || anchor == Anchor::TopRight {
        y_direction = 1.0;
    } else if anchor == Anchor::BottomLeft
        || anchor == Anchor::BottomCenter
        || anchor == Anchor::BottomRight
    {
        y_direction = -1.0;
    }
    if anchor == Anchor::TopLeft || anchor == Anchor::CenterLeft || anchor == Anchor::BottomLeft {
        x_direction = -1.0;
    } else if anchor == Anchor::TopRight
        || anchor == Anchor::CenterRight
        || anchor == Anchor::BottomRight
    {
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
        "bevy_novelgame_dialog::dialog_box::public::events::bds_event::ChangeCurrentTextArea": 
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
    mut commands: Commands,
    mut cb_query: Query<(Entity, &mut ChoiceBoxState), With<Current>>,
    db_query: Query<(Entity, &DialogBoxPhase, &Children), With<Current>>,
    ta_query: Query<(Entity, &TextArea, &GlobalTransform, &Sprite)>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    if let Ok((cbs_entity, mut cbs)) = cb_query.get_single_mut() {
        if cbs.choice_box_entity.is_none() {
            if let Ok((entity, dbp, children)) = db_query.get_single() {
                if *dbp != DialogBoxPhase::Fixed {
                    cbs.choice_box_entity = Some(entity)
                };
                for (ta_name, i) in cbs.text_area_names.iter().zip(0..) {
                    let target = cbs
                        .target_list
                        .get(i)
                        .map(|x| x.1.clone())
                        .unwrap_or_default();
                    let ron_base = ChoosenEvent {
                        choosen_event: target,
                        choice_box_state_entity: Some(cbs_entity),
                    };
                    let ron = write_ron(&app_type_registry, ron_base).unwrap_or_default();
                    for (ta_entity, ta, tf, sp) in ta_query.iter_many(children) {
                        if &ta.name == ta_name {
                            let wa = WaitInputGo {
                                ron: ron.clone(),
                                area: get_rect(tf, sp),
                            };
                            let se = Selective {
                                key_vector: cbs.select_vector,
                                number: i,
                            };
                            commands.entity(ta_entity).insert((wa, se));
                        }
                    }
                }
            }
        }
    }
}

fn get_rect(tf: &GlobalTransform, sp: &Sprite) -> Rect {
    let base_size = sp.custom_size.unwrap_or_default();
    let bottom_left = Vec2::new(tf.translation().x, tf.translation().y - base_size.y);
    let top_right = Vec2::new(bottom_left.x + base_size.x, tf.translation().y);
    Rect::from_corners(bottom_left, top_right)
}

pub fn close_choice_phase(
    mut commands: Commands,
    cbs_query: Query<&ChoiceBoxState>,
    mut db_query: Query<&mut DialogBoxPhase>,
    mut events: EventReader<BdsEvent>,
    mut gs_writer: EventWriter<GoSinking>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in events.read() {
        if let Some(ChoosenEvent {
            choosen_event: ce,
            choice_box_state_entity: Some(cbse),
        }) = event_wrapper.get_opt::<ChoosenEvent>()
        {
            if let Ok(next) = read_ron(&app_type_registry, ce) {
                commands.add(|w: &mut World| {
                    w.send_event(BdsEvent { value: next });
                });
            }
            if let Ok(cbs) = cbs_query.get(cbse) {
                let close = GoSinking {
                    target: cbs.choice_box_entity,
                    sink_type: cbs.sinkdown,
                };
                gs_writer.send(close);
                commands.entity(cbs.main_dialog_box).insert(Pending);
                if let Ok(mut dbp) = db_query.get_mut(cbs.main_dialog_box) {
                    *dbp = DialogBoxPhase::WaitToType;
                }
            }
        }
    }
}

pub fn reinstatement_external_entities(
    mut commands: Commands,
    cbs_query: Query<(Entity, &ChoiceBoxState)>,
    ta_query: Query<&TextArea>,
    children_query: Query<&Children>,
    mut sp_query: Query<&mut Sprite>,
    mut tf_query: Query<&mut Transform>,
) {
    for (state_entity, cbs) in &cbs_query {
        if let Some(cb_entity) = cbs.choice_box_entity {
            if let Ok(cb_children) = children_query.get(cb_entity) {
                if ta_query.iter_many(cb_children).next().is_none() {
                    let size = cbs.target_list.len() as f32;
                    let x_expand = cbs.background_scaling_per_button.x * size;
                    let y_expand = cbs.background_scaling_per_button.y * size;
                    let (x_dir, y_dir) = get_slide_direction(cbs.background_scaling_anchor);
                    if let Ok(mut sp) = sp_query.get_mut(cb_entity) {
                        sp.custom_size = sp
                            .custom_size
                            .map(|Vec2 { x, y }| Vec2::new(x - x_expand, y - y_expand));
                    }
                    for entity in &cbs.button_entities {
                        if let Ok(mut tf) = tf_query.get_mut(*entity) {
                            tf.translation.x -= x_dir * x_expand / 2.0;
                            tf.translation.y -= y_dir * y_expand / 2.0;
                        }
                    }
                    commands.entity(state_entity).despawn();
                }
            }
        }
    }
}
