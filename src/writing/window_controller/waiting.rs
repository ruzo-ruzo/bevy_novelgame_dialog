use super::super::*;
use bevy::render::view::RenderLayers;

#[derive(Component)]
pub(in crate::writing) struct Settled;

#[derive(Reflect, Default)]
pub(in crate::writing) struct BreakWait {
    pub writing_name: String,
    pub text_area_name: String,
}

#[derive(Reflect, Default, Clone)]
pub(in crate::writing) struct InputForSkipping {
    pub next_event_ron: String,
    pub writing_name: String,
    pub text_area_name: String,
}

pub(crate) struct MakeWigConfig<'a, S: AsRef<str>> {
    pub dialog_box_name: S,
    pub text_area_name: S,
    pub waiter_name: S,
    pub ron: S,
    pub type_registry: &'a AppTypeRegistry,
}

// SimpleWaitが発行された時のCurrentのTextAreaにBreakWaitを詰めたWaitInputGoを設定します。
// SimpleWaitが飛んでる間にCurrentのTextAreaが変更されていない事を期待しています。
#[allow(clippy::type_complexity)]
pub(in crate::writing) fn simple_wait(
    mut commands: Commands,
    mut dialog_query: Query<
        (Entity, &mut DialogBoxPhase, &DialogBox, &WaitBrakerStyle),
        With<Current>,
    >,
    w_icon_query: Query<(Entity, &WaitingIcon)>,
    text_area_query: Query<(Entity, &TextArea, &GlobalTransform, &Sprite, &ChildOf), With<Current>>,
    selected_query: Query<Entity, With<Selected>>,
    last_data: CurrentQuery,
    mut bds_reader: EventReader<BdsEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in bds_reader.read() {
        if event_wrapper.get::<SimpleWait>() != Some(SimpleWait) {
            continue;
        }
        for (mw_entity, mut ws, DialogBox { name: db_name }, wbs) in &mut dialog_query {
            for (ta_entity, ta, tb_tf, tb_sp, parent) in &text_area_query {
                if parent.parent() == mw_entity {
                    let ron = write_ron(
                        &type_registry,
                        BreakWait {
                            writing_name: db_name.clone(),
                            text_area_name: ta.name.clone(),
                        },
                    )
                    .unwrap_or_default();
                    let mf_config = MakeWigConfig {
                        dialog_box_name: db_name,
                        text_area_name: &ta.name,
                        waiter_name: &"".to_string(),
                        ron: &ron,
                        type_registry: &type_registry,
                    };
                    let wig = if let WaitBrakerStyle::Input {
                        is_all_range_area: true,
                        ..
                    } = wbs
                    {
                        make_wig_for_skip_all_range(mf_config)
                    } else {
                        make_wig_for_skip(mf_config, tb_tf, tb_sp)
                    };
                    commands.entity(ta_entity).insert(wig);
                }
                let (_, last_char) = initialize_typing_data(&last_data, ta_entity);
                let ic_opt = w_icon_query.iter().find(|x| {
                    x.1.target_box_name == *db_name
                        && x.1.wait_for.contains(&WaitTarget::SimpleWaiting)
                });
                if let Some((ic_entity, _)) = ic_opt {
                    let time = last_char.timer.timer.remaining_secs();
                    let tt = TypingTimer {
                        timer: Timer::from_seconds(time, TimerMode::Once),
                    };
                    commands.entity(ic_entity).insert(tt);
                    commands.entity(ic_entity).insert(ChildOf(ta_entity));
                }
                for s_entity in &selected_query {
                    commands.entity(s_entity).remove::<Selected>();
                }
                commands.entity(ta_entity).insert(Selected);
            }
            *ws = DialogBoxPhase::WaitingAction;
        }
    }
}

pub(in crate::writing) fn restart_typing(
    mut commands: Commands,
    mut writing_query: Query<(&DialogBox, &mut DialogBoxPhase)>,
    text_area_query: Query<&TextArea>,
    mut icon_query: Query<(Entity, &mut Visibility, &mut WaitingIcon)>,
    mut bds_reader: EventReader<BdsEvent>,
) {
    for event_wrapper in bds_reader.read() {
        if let Some(BreakWait {
            writing_name: target_db_name,
            text_area_name: target_ta_name,
        }) = event_wrapper.get::<BreakWait>()
        {
            for (DialogBox { name: db_name }, mut phase) in &mut writing_query {
                for TextArea { name: ta_name } in &text_area_query {
                    if *db_name == target_db_name
                        && *ta_name == target_ta_name
                        && DialogBoxPhase::WaitingAction == *phase
                    {
                        *phase = DialogBoxPhase::Typing;
                        let ic_opt = icon_query
                            .iter_mut()
                            .find(|x| x.2.target_box_name == *db_name);
                        if let Some((ic_entity, mut ic_vis, _)) = ic_opt {
                            commands.entity(ic_entity).remove::<TypingStyle>();
                            commands.entity(ic_entity).remove::<TypingTimer>();
                            *ic_vis = Visibility::Hidden;
                        }
                    }
                }
            }
        }
    }
}

pub(in crate::writing) fn waiting_icon_setting(
    mut commands: Commands,
    w_icon_query: Query<(Entity, &WaitingIcon), Without<WritingStyle>>,
    wbs_query: Query<(&RenderLayers, &WaitBrakerStyle, &DialogBox)>,
) {
    for (layer, wbs, DialogBox { name: db_name }) in &wbs_query {
        if let WaitBrakerStyle::Input { .. } = wbs {
            if let Some((ic_entity, _)) = w_icon_query
                .iter()
                .find(|x| x.1.target_box_name == *db_name)
            {
                commands.entity(ic_entity).insert((
                    WritingStyle::Put,
                    layer.clone(),
                    Visibility::Hidden,
                ));
            }
        }
    }
}

// CurrentQueryとiconのQueryがTransform取りあってるためWithoutかけてます
#[allow(clippy::type_complexity)]
pub(in crate::writing) fn settle_wating_icon(
    mut commands: Commands,
    window_query: Query<(Entity, &DialogBoxPhase, &WaitBrakerStyle, &DialogBox)>,
    text_box_query: Query<(Entity, &ChildOf, &TypeTextConfig), (With<TextArea>, With<Current>)>,
    mut float_icon_query: Query<
        (Entity, &mut Transform, &mut WaitingIcon),
        (
            Without<MessageTextLine>,
            Without<MessageTextChar>,
            Without<Settled>,
        ),
    >,
    settle_icon_query: Query<(Entity, &WaitingIcon), With<Settled>>,
    last_data: CurrentQuery,
) {
    for (mw_entity, ws, wbs, DialogBox { name: db_name }) in &window_query {
        if let WaitBrakerStyle::Input {
            is_icon_moving_to_last: move_flag,
            ..
        } = wbs
        {
            if *ws == DialogBoxPhase::WaitingAction {
                for (ic_entity, mut ic_tf, wi) in &mut float_icon_query {
                    if wi.target_box_name == *db_name {
                        if let Some((tb_entity, _, config)) = text_box_query
                            .iter()
                            .find(|(_, p, _)| p.parent() == mw_entity)
                        {
                            let (_, lc) = initialize_typing_data(&last_data, tb_entity);
                            if *move_flag {
                                ic_tf.translation =
                                    Vec3::new(lc.pos.x + config.base_size, lc.pos.y, 1.);
                            }
                        }
                        commands.entity(ic_entity).insert(Settled);
                    }
                }
            } else {
                for (ic_entity, wi) in &settle_icon_query {
                    if wi.target_box_name == *db_name {
                        commands.entity(ic_entity).remove::<Settled>();
                    }
                }
            }
        }
    }
}

// WaitingAction中のみ作動します。（Feeding中にはskip_feedingが作動します）
// InputForSkippingを受け取り、すでに全ての文字の表示が終わっていれば内部のronをBSDEventとして発行します。
// 文字の表示が終わっていなければ、WaitInputGo内にInputForSkippingを詰めてTextAreaにつけ直します。
// その後当該TextArea内の文字を強制的に表示し終えます。
// これをトリガーするInputForSkippingはSimpleWait経由で発行されていることが期待されています。
// ここで参照しているため、全ての文字は表示し終わった時点でTypingStyle::Putを持つ必要があります。
#[allow(clippy::type_complexity)]
pub(in crate::writing) fn skip_typing_or_next(
    mut commands: Commands,
    mut waiting_text_query: Query<
        (
            Entity,
            &mut Visibility,
            &mut Transform,
            &ChildOf,
            &mut TypingTimer,
        ),
        With<MessageTextChar>,
    >,
    mut typing_texts_query: Query<(Entity, &mut TypingStyle, &ChildOf), With<MessageTextChar>>,
    writing_query: Query<(&DialogBox, &DialogBoxPhase, &WaitBrakerStyle)>,
    text_area_query: Query<(Entity, &TextArea, &GlobalTransform, &Sprite)>,
    line_query: Query<(Entity, &ChildOf), With<MessageTextLine>>,
    mut icon_query: Query<
        (Entity, &mut Visibility),
        (
            With<WaitingIcon>,
            With<TypingTimer>,
            Without<MessageTextChar>,
        ),
    >,
    mut bds_reader: EventReader<BdsEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in bds_reader.read() {
        if let Some(InputForSkipping {
            next_event_ron: ron,
            writing_name: target_db_name,
            text_area_name: target_ta_name,
        }) = event_wrapper.get::<InputForSkipping>()
        {
            let db_opt = writing_query.iter().find(|x| x.0.name == target_db_name);
            let ta_opt = text_area_query.iter().find(|x| x.1.name == target_ta_name);
            if let (Some((db, phase, wbs)), Some((ta_entity, ta, tb_tf, tb_sp))) = (db_opt, ta_opt)
            {
                if *phase != DialogBoxPhase::WaitingAction {
                    return;
                }
                let mut typed_count = 0usize;
                let mut text_count = 0usize;
                for (text_entity, ts, t_parent) in &mut typing_texts_query {
                    if line_query.get(t_parent.parent()).map(|x| x.1.parent()) == Ok(ta_entity) {
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
                if text_count <= typed_count {
                    for (ic_entity, mut ic_vis) in &mut icon_query {
                        *ic_vis = Visibility::Hidden;
                        commands.entity(ic_entity).remove::<TypingTimer>();
                        commands.entity(ic_entity).remove::<TypingStyle>();
                    }
                    if let Ok(ref_value) = read_ron(&type_registry, ron.clone()) {
                        commands.queue(|w: &mut World| {
                            w.send_event(BdsEvent { value: ref_value });
                        })
                    }
                } else {
                    let mf_config = MakeWigConfig {
                        dialog_box_name: &db.name,
                        text_area_name: &ta.name,
                        waiter_name: &"Waiting Next Type".to_string(),
                        ron: &ron,
                        type_registry: &type_registry,
                    };
                    let wig = if let WaitBrakerStyle::Input {
                        is_all_range_area: true,
                        ..
                    } = wbs
                    {
                        make_wig_for_skip_all_range(mf_config)
                    } else {
                        make_wig_for_skip(mf_config, tb_tf, tb_sp)
                    };
                    commands.entity(ta_entity).insert(wig);
                    for (_, mut ic_vis) in &mut icon_query {
                        *ic_vis = Visibility::Inherited;
                    }
                }
                for (text_entity, mut t_vis, mut tf, t_parent, mut tt) in &mut waiting_text_query {
                    if line_query.get(t_parent.parent()).map(|x| x.1.parent()) == Ok(ta_entity) {
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
}

pub(in crate::writing) fn skip_feeding(
    mut commands: Commands,
    mut writing_query: Query<(&DialogBox, &mut DialogBoxPhase), With<Current>>,
    text_area_query: Query<(Entity, &TextArea)>,
    line_query: Query<(Entity, &ChildOf), With<MessageTextLine>>,
    mut bds_reader: EventReader<BdsEvent>,
) {
    for event_wrapper in bds_reader.read() {
        if let Some(InputForSkipping {
            writing_name: target_db_name,
            text_area_name: target_ta_name,
            ..
        }) = event_wrapper.get::<InputForSkipping>()
        {
            let db_opt = writing_query
                .iter_mut()
                .find(|x| x.0.name == target_db_name);
            let ta_opt = text_area_query.iter().find(|x| x.1.name == target_ta_name);
            if let (Some((_, mut phase)), Some((ta_entity, _))) = (db_opt, ta_opt) {
                if *phase == DialogBoxPhase::Feeding {
                    for (l_entity, l_parent) in &line_query {
                        if l_parent.parent() == ta_entity {
                            commands.entity(l_entity).despawn();
                        }
                    }
                    *phase = DialogBoxPhase::Typing;
                }
            }
        }
    }
}

pub(in crate::writing) fn make_wig_for_skip<S: AsRef<str>>(
    config: MakeWigConfig<S>,
    tb_tf: &GlobalTransform,
    tb_sp: &Sprite,
) -> WaitInputGo {
    let base_size = tb_sp.custom_size.unwrap_or_default();
    let bottom_left = Vec2::new(tb_tf.translation().x, tb_tf.translation().y - base_size.y);
    let top_right = Vec2::new(bottom_left.x + base_size.x, tb_tf.translation().y);
    let ron_ifs_opt = write_ron(
        config.type_registry,
        InputForSkipping {
            next_event_ron: config.ron.as_ref().to_string(),
            writing_name: config.dialog_box_name.as_ref().to_string(),
            text_area_name: config.text_area_name.as_ref().to_string(),
        },
    );
    WaitInputGo {
        ron: ron_ifs_opt.unwrap_or_default(),
        area: Rect::from_corners(bottom_left, top_right),
        waiter_name: config.waiter_name.as_ref().to_string(),
    }
}

pub(in crate::writing) fn make_wig_for_skip_all_range<S: AsRef<str>>(
    config: MakeWigConfig<S>,
) -> WaitInputGo {
    let bottom_left = Vec2::new(f32::MIN, f32::MIN);
    let top_right = Vec2::new(f32::MAX, f32::MAX);
    let ron_ifs_opt = write_ron(
        config.type_registry,
        InputForSkipping {
            next_event_ron: config.ron.as_ref().to_string(),
            writing_name: config.dialog_box_name.as_ref().to_string(),
            text_area_name: config.text_area_name.as_ref().to_string(),
        },
    );
    WaitInputGo {
        ron: ron_ifs_opt.unwrap_or_default(),
        area: Rect::from_corners(bottom_left, top_right),
        waiter_name: config.waiter_name.as_ref().to_string(),
    }
}

pub(in crate::writing) fn hide_waiting_icon(
    mut icon_query: Query<(&WaitingIcon, &mut Visibility)>,
    writing_query: Query<(&DialogBox, &DialogBoxPhase)>,
) {
    if let Ok((icon, mut vis)) = icon_query.single_mut() {
        let box_exists = writing_query
            .iter()
            .find(|x| x.0.name == icon.target_box_name);
        if let Some((_, phase)) = box_exists {
            if *phase != DialogBoxPhase::SinkingDown {
                return;
            }
        }
        *vis = Visibility::Hidden;
    }
}
