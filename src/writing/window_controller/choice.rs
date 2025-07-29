use super::*;
use crate::writing::*;
use bevy::render::view::RenderLayers;

// Todo: 名前の重複を防ぐ機構を入れた方がいいかもしれない
#[derive(Component)]
pub(in crate::writing) struct ChoiceBoxState {
    main_writing_name: String,
    text_area_names: Vec<String>,
    choice_box_name: String,
    target_list: Vec<(String, String)>,
    select_vector: SelectVector,
    sinkdown: SinkDownType,
    background_scaling_per_button: Vec2,
    background_scaling_anchor: Anchor,
}

#[derive(Event, Default, Reflect)]
pub(in crate::writing) struct SetupChoice {
    target_list: Vec<(String, String)>,
}

#[derive(Reflect, Default)]
pub(in crate::writing) struct ChoosenEvent {
    pub choosen_event: String,
    pub choice_box_name: String,
}

#[derive(Component)]
pub(in crate::writing) struct Choosable;

#[allow(clippy::type_complexity)]
pub(in crate::writing) fn open_choice_box(
    mut commands: Commands,
    mut db_query: Query<
        (&ChoiceBoxConfig, &mut DialogBoxPhase, &Children, &DialogBox),
        With<Current>,
    >,
    mut vis_query: Query<&mut Visibility>,
    cc_query: Query<Entity, (With<ChoiceBoxState>, With<Current>)>,
    cb_query: Query<(Entity, &ChoiceButton)>,
    bg_query: Query<(Entity, &DialogBoxBackground)>,
    mut sp_query: Query<&mut Sprite>,
    mut tf_query: Query<&mut Transform>,
    mut events: EventReader<BdsEvent>,
    setup_config: Res<SetupConfig>,
    mut ow_event: EventWriter<OpenDialog>,
) {
    for event_wrapper in events.read() {
        if let Some(SetupChoice { target_list: tl }) = event_wrapper.get::<SetupChoice>() {
            if let Ok((cbc, mut dbs, children, db)) = db_query.get_single_mut() {
                let bg_opt = bg_query
                    .iter()
                    .find(|x| x.1.writing_name == cbc.choice_box_name);
                let background_entity = if let Some((entity, _)) = bg_opt {
                    entity
                } else {
                    commands.spawn(Sprite::default()).id()
                };
                let button_entities = cb_query
                    .iter()
                    .filter(|x| x.1.target_box_name == cbc.choice_box_name)
                    .map(|x| x.0)
                    .collect::<Vec<_>>();
                let culled_tas = &cbc.button_text_areas[0..tl.len()];
                let ta_names = culled_tas
                    .iter()
                    .map(|c| c.area_name.clone())
                    .collect::<Vec<_>>();
                let cs = ChoiceBoxState {
                    main_writing_name: db.name.clone(),
                    choice_box_name: cbc.choice_box_name.clone(),
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
                    .insert(cs)
                    .add_children(&button_entities);
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
                for (i, entity) in button_entities.iter().enumerate() {
                    if let Ok(mut vis) = vis_query.get_mut(*entity) {
                        if i >= tl.len() {
                            *vis = Visibility::Hidden;
                        } else {
                            *vis = Visibility::Inherited;
                        }
                    }
                    commands
                        .entity(*entity)
                        .insert(RenderLayers::layer(setup_config.render_layer.into()));
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
                let opening_event = OpenDialog {
                    writing_name: cbc.choice_box_name.clone(),
                    raw_orders: make_choice_order(&tl, &cbc.choice_box_name, &ta_names),
                    popup: cbc.popup,
                    text_area_configs: slided_text_area_configs,
                    wait_breaker: WaitBrakerStyle::Auto {
                        wait_sec: cbc.wait_to_sink,
                    },
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
    writing_name: &str,
    text_area_names: &[String],
) -> Option<Vec<Order>> {
    let header = r#"<script>{
        "bevy_novelgame_dialog::writing::events::bds::ChangeCurrentTextArea": 
        (target_writing_name: ""#;
    let midpoint = r#"",  next_current_text_area_name: ""#;
    let footer = r#"",),}</script>"#;
    let mut script = String::new();
    for ((text, _), name) in target_list.iter().zip(text_area_names.iter()) {
        script.push_str(&format!("{header}{writing_name}{midpoint}{name}{footer}"));
        script.push_str(text);
    }
    Some(parse_script(&script, &[""], ""))
}

pub(in crate::writing) fn setup_choice(
    mut commands: Commands,
    cb_query: Query<(Entity, &ChoiceBoxState, &Children, &DialogBoxPhase), With<Current>>,
    ta_query: Query<(Entity, &TextArea, &GlobalTransform, &Sprite), Without<Selective>>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    if let Ok((cb_entity, cbs, children, dbp)) = cb_query.get_single() {
        if *dbp != DialogBoxPhase::Typing {
            return;
        }
        for (i, ta_name) in cbs.text_area_names.iter().enumerate() {
            let target = cbs
                .target_list
                .get(i)
                .map(|x| x.1.clone())
                .unwrap_or_default();
            let ron_base = ChoosenEvent {
                choosen_event: target,
                choice_box_name: cbs.choice_box_name.clone(),
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
        commands.entity(cb_entity).insert(Choosable);
    }
}

fn get_rect(tf: &GlobalTransform, sp: &Sprite) -> Rect {
    let base_size = sp.custom_size.unwrap_or_default();
    let bottom_left = Vec2::new(tf.translation().x, tf.translation().y - base_size.y);
    let top_right = Vec2::new(bottom_left.x + base_size.x, tf.translation().y);
    Rect::from_corners(bottom_left, top_right)
}

pub(in crate::writing) fn close_choice_phase(
    mut commands: Commands,
    cbs_query: Query<&ChoiceBoxState>,
    mut db_query: Query<(Entity, &DialogBox, &mut DialogBoxPhase)>,
    mut events: EventReader<BdsEvent>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in events.read() {
        if let Some(ChoosenEvent {
            choosen_event: ce,
            choice_box_name: cb_name,
        }) = event_wrapper.get::<ChoosenEvent>()
        {
            if let Ok(next) = read_ron(&app_type_registry, ce) {
                commands.queue(|w: &mut World| {
                    w.send_event(BdsEvent { value: next });
                });
            }
            if let Some(cbs) = cbs_query.iter().find(|x| x.choice_box_name == cb_name) {
                for (db_entity, db, mut dbp) in &mut db_query {
                    if db.name == cbs.main_writing_name {
                        let close = BdsEvent {
                            value: Box::new(SinkDownWindow {
                                sink_type: cbs.sinkdown,
                            }),
                        };
                        commands.queue(|w: &mut World| {
                            w.send_event(close);
                        });
                        commands.entity(db_entity).insert(Pending);
                        *dbp = DialogBoxPhase::WaitToType;
                    }
                }
            }
        }
    }
}

pub(in crate::writing) fn reinstatement_external_entities(
    mut commands: Commands,
    cbs_query: Query<(Entity, &ChoiceBoxState), With<Choosable>>,
    cb_query: Query<(Entity, &ChoiceButton)>,
    ta_query: Query<&TextArea>,
    children_query: Query<&Children>,
    mut sp_query: Query<&mut Sprite>,
    mut tf_query: Query<&mut Transform>,
) {
    for (state_entity, cbs) in &cbs_query {
        if let Ok(cb_children) = children_query.get(state_entity) {
            if ta_query.iter_many(cb_children).next().is_none() {
                let size = cbs.target_list.len() as f32;
                let x_expand = cbs.background_scaling_per_button.x * size;
                let y_expand = cbs.background_scaling_per_button.y * size;
                let (x_dir, y_dir) = get_slide_direction(cbs.background_scaling_anchor);
                if let Ok(mut sp) = sp_query.get_mut(state_entity) {
                    sp.custom_size = sp
                        .custom_size
                        .map(|Vec2 { x, y }| Vec2::new(x - x_expand, y - y_expand));
                }
                let cb_entities = cb_query
                    .iter()
                    .filter(|x| x.1.target_box_name == cbs.choice_box_name);
                for (entity, _) in cb_entities {
                    if let Ok(mut tf) = tf_query.get_mut(entity) {
                        tf.translation.x -= x_dir * x_expand / 2.0;
                        tf.translation.y -= y_dir * y_expand / 2.0;
                    }
                }
                commands.entity(state_entity).remove::<ChoiceBoxState>();
                commands.entity(state_entity).remove::<Choosable>();
            }
        }
    }
}
