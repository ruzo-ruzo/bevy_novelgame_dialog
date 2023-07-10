use super::super::*;
use bevy::render::view::RenderLayers;

#[derive(Component, Debug)]
pub struct WaitingIcon {
    is_settled: bool,
}

#[derive(Reflect, Default, Debug)]
pub struct InputForFeeding {
    pub target_text_box: Option<Entity>,
}

#[derive(Reflect, Default, Clone, Debug)]
pub struct InputForSkipping {
    pub next_event_ron: String,
    pub target_text_box: Option<Entity>,
}

#[derive(Reflect, Default, Debug, PartialEq)]
pub struct SimpleWait;

#[derive(Reflect, Default, Debug)]
pub struct BreakWait {
    pub target_text_box: Option<Entity>,
}

#[allow(clippy::type_complexity)]
pub fn simple_wait(
    mut commands: Commands,
    mut window_query: Query<(Entity, &mut WindowState, &WaitBrakerStyle), With<MessageWindow>>,
    text_box_query: Query<
        (Entity, &GlobalTransform, &Sprite, &Parent),
        (With<Current>, With<TextBox>),
    >,
    mut bms_reader: EventReader<BMSEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in bms_reader.iter() {
        if event_wrapper.get_opt::<SimpleWait>() == Some(SimpleWait) {
            for (mw_entity, mut ws, wbs) in &mut window_query {
                if let WaitBrakerStyle::Input { .. } = wbs {
                    for (tb_entity, tb_tf, tb_sp, parent) in &text_box_query {
                        if parent.get() == mw_entity {
                            let ron = write_ron(
                                &type_registry,
                                BreakWait {
                                    target_text_box: Some(tb_entity),
                                },
                            )
                            .unwrap_or_default();
                            let wig =
                                make_wig_for_skip(tb_entity, tb_tf, tb_sp, ron, &type_registry);
                            commands.entity(tb_entity).insert(wig);
                        }
                    }
                    *ws = WindowState::Waiting;
                }
            }
        }
    }
}

pub fn restart_typing(
    mut window_query: Query<(Entity, &mut WindowState, &WaitBrakerStyle), With<MessageWindow>>,
    text_box_query: Query<&Parent, With<TextBox>>,
    mut icon_query: Query<(&mut Visibility, &mut WaitingIcon)>,
    mut bms_reader: EventReader<BMSEvent>,
) {
    for event_wrapper in bms_reader.iter() {
        if let Some(BreakWait {
            target_text_box: Some(tb_entity),
        }) = event_wrapper.get_opt::<BreakWait>()
        {
            for (mw_entity, mut ws, wbs) in &mut window_query {
                if let Ok(tb_parent) = text_box_query.get(tb_entity) {
                    if tb_parent.get() == mw_entity && WindowState::Waiting == *ws {
                        *ws = WindowState::Typing;
                    }
                }
                if let WaitBrakerStyle::Input {
                    icon_entity: Some(ic_entity),
                    ..
                } = wbs
                {
                    if let Ok((mut ic_vis, mut wi)) = icon_query.get_mut(*ic_entity) {
                        *wi = WaitingIcon { is_settled: false };
                        *ic_vis = Visibility::Hidden;
                    }
                }
            }
        }
    }
}

pub fn waiting_icon_setting(
    mut commands: Commands,
    wbs_query: Query<(&RenderLayers, &WaitBrakerStyle)>,
    no_tag: Query<Entity, Without<WaitingIcon>>,
) {
    for (layer, wbs) in &wbs_query {
        if let WaitBrakerStyle::Input {
            icon_entity: Some(ic_entity),
            ..
        } = wbs
        {
            for no_tag_entity in &no_tag {
                if *ic_entity == no_tag_entity {
                    commands.entity(*ic_entity).insert((
                        WaitingIcon { is_settled: false },
                        WritingStyle::Put,
                        *layer,
                        Visibility::Hidden,
                    ));
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn settle_wating_icon(
    window_query: Query<(Entity, &WindowState, &WaitBrakerStyle), With<MessageWindow>>,
    text_box_query: Query<(Entity, &Parent, &TypeTextConfig), With<TextBox>>,
    mut icon_query: Query<
        (&mut Transform, &mut WaitingIcon),
        (Without<MessageTextLine>, Without<MessageTextChar>),
    >,
    last_data: LastTextData,
) {
    for (mw_entity, ws, wbs) in &window_query {
        if let WaitBrakerStyle::Input {
            icon_entity: Some(ic_entity),
            is_icon_moving_to_last: move_flag,
        } = wbs
        {
            if let Ok((mut ic_tf, mut wi)) = icon_query.get_mut(*ic_entity) {
                let WaitingIcon {
                    is_settled: settled,
                } = &mut *wi;
                if *ws == WindowState::Waiting {
                    if *settled {
                        return;
                    }
                    if let Some((tb_entity, _, config)) =
                        text_box_query.iter().find(|(_, p, _)| p.get() == mw_entity)
                    {
                        let (_, _, last_x, last_y, _) =
                            initialize_typing_data(&last_data, tb_entity);
                        if *move_flag {
                            ic_tf.translation =
                                Vec3::new(last_x + config.text_style.font_size, last_y, 1.);
                        }
                    }
                    *settled = true;
                } else {
                    *settled = false;
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn skip_typing_or_next(
    mut commands: Commands,
    mut waiting_text_query: Query<
        (Entity, &mut Visibility, &mut Transform, &Parent),
        With<MessageTextChar>,
    >,
    mut typing_texts_query: Query<(Entity, &mut TypingStyle, &Parent), With<MessageTextChar>>,
    window_query: Query<&WindowState, With<MessageWindow>>,
    text_box_query: Query<(&GlobalTransform, &Sprite, &Parent), With<TextBox>>,
    line_query: Query<(Entity, &Parent), With<MessageTextLine>>,
    mut icon_query: Query<(Entity, &mut Visibility), (With<WaitingIcon>, Without<MessageTextChar>)>,
    mut bms_reader: EventReader<BMSEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in bms_reader.iter() {
        if let Some(InputForSkipping {
            next_event_ron: ron,
            target_text_box: Some(tb_entity),
        }) = event_wrapper.get_opt::<InputForSkipping>()
        {
            if let Ok((_, _, parent)) = text_box_query.get(tb_entity) {
                if window_query.get(parent.get()) != Ok(&WindowState::Waiting) {
                    return;
                }
            }
            let mut typed_count = 0usize;
            let mut text_count = 0usize;
            for (text_entity, ts, t_parent) in &mut typing_texts_query {
                if line_query.get(t_parent.get()).map(|x| x.1.get()) == Ok(tb_entity) {
                    match *ts {
                        TypingStyle::Typed => {
                            typed_count += 1;
                        }
                        _ => {
                            commands.entity(text_entity).remove::<TypingStyle>();
                            commands.entity(text_entity).insert(TypingStyle::Typed);
                        }
                    }
                    text_count += 1;
                }
            }
            for (ic_entity, mut ic_vis) in &mut icon_query {
                *ic_vis = Visibility::Inherited;
                commands.entity(ic_entity).remove::<TypingTimer>();
            }
            if let Ok((tb_tf, tb_sp, _)) = text_box_query.get(tb_entity) {
                if text_count == typed_count {
                    if let Ok(ref_value) = read_ron(&type_registry, ron.clone()) {
                        commands.add(|w: &mut World| {
                            w.send_event(BMSEvent { value: ref_value });
                        })
                    }
                } else {
                    let wig = make_wig_for_skip(tb_entity, tb_tf, tb_sp, ron, &type_registry);
                    commands.entity(tb_entity).insert(wig);
                }
            }
            for (text_entity, mut t_vis, mut tf, t_parent) in &mut waiting_text_query {
                if line_query.get(t_parent.get()).map(|x| x.1.get()) == Ok(tb_entity) {
                    tf.scale = Vec3::ONE;
                    *t_vis = Visibility::Inherited;
                    commands.entity(text_entity).remove::<TypingTimer>();
                    commands.entity(text_entity).insert(TypingStyle::Typed);
                }
            }
        }
    }
}

pub fn skip_feeding(
    mut commands: Commands,
    mut window_query: Query<&mut WindowState, With<MessageWindow>>,
    text_box_query: Query<&Parent, With<TextBox>>,
    line_query: Query<(Entity, &Parent), With<MessageTextLine>>,
    mut bms_reader: EventReader<BMSEvent>,
) {
    for event_wrapper in bms_reader.iter() {
        if let Some(InputForSkipping {
            next_event_ron: _,
            target_text_box: Some(tb_entity),
        }) = event_wrapper.get_opt::<InputForSkipping>()
        {
            if let Ok(tb_parent) = text_box_query.get(tb_entity) {
                if let Ok(mut ws) = window_query.get_mut(tb_parent.get()) {
                    if *ws == WindowState::Feeding {
                        for (l_entity, l_parent) in &line_query {
                            if l_parent.get() == tb_entity {
                                commands.entity(l_entity).despawn_recursive();
                            }
                        }
                        *ws = WindowState::Typing;
                        // } else {
                        // let bmse = BMSEvent{
                        // value: Box::new(ifs.clone())
                        // };
                        // commands.add(move |w: &mut World| {
                        // w.send_event(bmse);
                        // });
                    }
                }
            }
        }
    }
}

pub fn make_wig_for_skip<S: AsRef<str>>(
    tb_entity: Entity,
    tb_tf: &GlobalTransform,
    tb_sp: &Sprite,
    ron: S,
    type_registry: &AppTypeRegistry,
) -> WaitInputGo {
    let base_size = tb_sp.custom_size.unwrap_or_default();
    let bottom_left = Vec2::new(tb_tf.translation().x, tb_tf.translation().y - base_size.y);
    let top_right = Vec2::new(bottom_left.x + base_size.x, tb_tf.translation().y);
    let ron_ifs_opt = write_ron(
        type_registry,
        InputForSkipping {
            next_event_ron: ron.as_ref().to_string(),
            target_text_box: Some(tb_entity),
        },
    );
    WaitInputGo {
        ron: ron_ifs_opt.unwrap_or_default(),
        area: Rect::from_corners(bottom_left, top_right),
    }
}
