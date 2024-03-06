use super::super::*;
use bevy::render::view::RenderLayers;
// use bevy::utils::Duration;

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
    mut window_query: Query<(Entity, &mut DialogBoxState, &WaitBrakerStyle), With<DialogBox>>,
    text_box_query: Query<
        (Entity, &GlobalTransform, &Sprite, &Parent),
        (With<Current>, With<TextArea>),
    >,
    selected_query: Query<Entity, With<Selected>>,
    last_data: LastTextData,
    mut bds_reader: EventReader<BdsEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in bds_reader.read() {
        if event_wrapper.get_opt::<SimpleWait>() == Some(SimpleWait) {
            for (mw_entity, mut ws, wbs) in &mut window_query {
                if let WaitBrakerStyle::Input {
                    icon_entity: icon_opt,
                    ..
                } = wbs
                {
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
                        let (_, _, _, _, last_timer) =
                            initialize_typing_data(&last_data, tb_entity);
                        if let Some(ic_entity) = icon_opt {
                            let time = last_timer.timer.remaining_secs();
                            let tt = TypingTimer {
                                timer: Timer::from_seconds(time, TimerMode::Once),
                            };
                            commands.entity(*ic_entity).insert(tt);
                            commands.entity(*ic_entity).set_parent(tb_entity);
                        }
                        for s_entity in &selected_query {
                            commands.entity(s_entity).remove::<Selected>();
                        }
                        commands.entity(tb_entity).insert(Selected);
                    }
                    *ws = DialogBoxState::WaitingAction;
                }
            }
        }
    }
}

pub fn restart_typing(
    mut window_query: Query<(Entity, &mut DialogBoxState, &WaitBrakerStyle), With<DialogBox>>,
    text_box_query: Query<&Parent, With<TextArea>>,
    mut icon_query: Query<(&mut Visibility, &mut WaitingIcon)>,
    mut bds_reader: EventReader<BdsEvent>,
) {
    for event_wrapper in bds_reader.read() {
        if let Some(BreakWait {
            target_text_box: Some(tb_entity),
        }) = event_wrapper.get_opt::<BreakWait>()
        {
            for (mw_entity, mut ws, wbs) in &mut window_query {
                if let Ok(tb_parent) = text_box_query.get(tb_entity) {
                    if tb_parent.get() == mw_entity && DialogBoxState::WaitingAction == *ws {
                        *ws = DialogBoxState::Typing;
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
    window_query: Query<(Entity, &DialogBoxState, &WaitBrakerStyle), With<DialogBox>>,
    text_box_query: Query<(Entity, &Parent, &TypeTextConfig), With<TextArea>>,
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
                if *ws == DialogBoxState::WaitingAction {
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
        (
            Entity,
            &mut Visibility,
            &mut Transform,
            &Parent,
            &mut TypingTimer,
        ),
        With<MessageTextChar>,
    >,
    mut typing_texts_query: Query<(Entity, &mut TypingStyle, &Parent), With<MessageTextChar>>,
    window_query: Query<&DialogBoxState, With<DialogBox>>,
    text_box_query: Query<(&GlobalTransform, &Sprite, &Parent), With<TextArea>>,
    line_query: Query<(Entity, &Parent), With<MessageTextLine>>,
    mut icon_query: Query<(Entity, &mut Visibility), (With<WaitingIcon>, Without<MessageTextChar>)>,
    mut bds_reader: EventReader<BdsEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in bds_reader.read() {
        if let Some(InputForSkipping {
            next_event_ron: ron,
            target_text_box: Some(tb_entity),
        }) = event_wrapper.get_opt::<InputForSkipping>()
        {
            if let Ok((_, _, parent)) = text_box_query.get(tb_entity) {
                if window_query.get(parent.get()) != Ok(&DialogBoxState::WaitingAction) {
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
                            w.send_event(BdsEvent { value: ref_value });
                        })
                    }
                } else {
                    let wig = make_wig_for_skip(tb_entity, tb_tf, tb_sp, ron, &type_registry);
                    commands.entity(tb_entity).insert(wig);
                }
            }
            for (text_entity, mut t_vis, mut tf, t_parent, mut tt) in &mut waiting_text_query {
                if line_query.get(t_parent.get()).map(|x| x.1.get()) == Ok(tb_entity) {
                    tf.scale = Vec3::ONE;
                    *t_vis = Visibility::Inherited;
                    let rem = tt.timer.duration();
                    tt.timer.tick(rem);
                    commands.entity(text_entity).insert(TypingStyle::Typed);
                }
            }
        }
    }
}

pub fn skip_feeding(
    mut commands: Commands,
    mut window_query: Query<&mut DialogBoxState, With<DialogBox>>,
    text_box_query: Query<&Parent, With<TextArea>>,
    line_query: Query<(Entity, &Parent), With<MessageTextLine>>,
    mut bds_reader: EventReader<BdsEvent>,
) {
    for event_wrapper in bds_reader.read() {
        if let Some(InputForSkipping {
            next_event_ron: _,
            target_text_box: Some(tb_entity),
        }) = event_wrapper.get_opt::<InputForSkipping>()
        {
            if let Ok(tb_parent) = text_box_query.get(tb_entity) {
                if let Ok(mut ws) = window_query.get_mut(tb_parent.get()) {
                    if *ws == DialogBoxState::Feeding {
                        for (l_entity, l_parent) in &line_query {
                            if l_parent.get() == tb_entity {
                                commands.entity(l_entity).despawn_recursive();
                            }
                        }
                        *ws = DialogBoxState::Typing;
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
