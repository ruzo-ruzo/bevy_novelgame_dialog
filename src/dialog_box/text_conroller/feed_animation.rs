use super::super::window_controller::waiting::*;
use super::super::*;

#[derive(Event, Debug)]
pub struct FeedWaitingEvent {
    pub target_window: Entity,
    pub wait_sec: f32,
    pub last_pos: Vec2,
}

#[derive(Event, Debug)]
pub struct StartFeedingEvent;

#[derive(Component, Debug)]
pub struct WaitFeedingTrigger {
    pub timer: Timer,
}

#[derive(Component, Debug)]
pub struct ScrollFeed {
    pub line_per_sec: f32,
    pub count: usize,
}

#[allow(clippy::type_complexity)]
pub fn setup_feed_starter(
    mut commands: Commands,
    dialog_box_query: Query<(Entity, &WaitBrakerStyle, &DialogBox)>,
    text_box_query: Query<(Entity, &TextArea, &Parent, &GlobalTransform, &Sprite)>,
    w_icon_query: Query<(Entity, &WaitingIcon)>,
    current_query: Query<&Current>,
    mut waitting_event: EventReader<FeedWaitingEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event in waitting_event.read() {
        for (w_entity, wbs, DialogBox { name: db_name }) in &dialog_box_query {
            for (tb_entity, ta, parent, tb_tf, tb_sp) in &text_box_query {
                if event.target_window == w_entity && w_entity == parent.get() {
                    if current_query.get(tb_entity).is_err() {
                        commands.entity(tb_entity).remove::<Selected>();
                        continue;
                    }
                    match wbs {
                        WaitBrakerStyle::Auto { wait_sec: break_ws } => {
                            commands.entity(tb_entity).insert(WaitFeedingTrigger {
                                timer: Timer::from_seconds(
                                    event.wait_sec + break_ws,
                                    TimerMode::Once,
                                ),
                            });
                        }
                        WaitBrakerStyle::Input {
                            is_all_range_area: is_all_range,
                            ..
                        } => {
                            let icon_opt = w_icon_query
                                .iter()
                                .find(|x| *db_name == x.1.target_box_name);
                            if let Some((ic_entity, _)) = icon_opt {
                                let tt = TypingTimer {
                                    timer: Timer::from_seconds(event.wait_sec, TimerMode::Once),
                                };
                                commands.entity(ic_entity).insert(tt);
                                commands.entity(ic_entity).set_parent(tb_entity);
                            }
                            let ron_iff = write_ron(
                                &type_registry,
                                InputForFeeding {
                                    dialog_box_name: db_name.clone(),
                                    text_area_name: ta.name.clone(),
                                },
                            )
                            .unwrap_or_default();
                            let wig = if *is_all_range {
                                make_wig_for_skip_all_range(
                                    db_name,
                                    &ta.name,
                                    &ron_iff,
                                    &type_registry,
                                )
                            } else {
                                make_wig_for_skip(
                                    db_name,
                                    &ta.name,
                                    tb_tf,
                                    tb_sp,
                                    &ron_iff,
                                    &type_registry,
                                )
                            };
                            commands.entity(tb_entity).insert(wig);
                            commands.entity(tb_entity).insert(Selected);
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn trigger_feeding_by_event(
    mut commands: Commands,
    mut line_query: Query<(Entity, &Parent), With<MessageTextLine>>,
    mut dialog_box_query: Query<(&DialogBox, &mut DialogBoxPhase)>,
    text_area_query: Query<(Entity, &TextArea, &FeedingStyle), With<Current>>,
    mut icon_query: Query<(Entity, &mut Visibility), (With<WaitingIcon>, Without<MessageTextChar>)>,
    mut start_feeding_event: EventWriter<StartFeedingEvent>,
    mut events: EventReader<BdsEvent>,
) {
    for event_wrapper in events.read() {
        if let Some(InputForFeeding {
            dialog_box_name: target_db_name,
            text_area_name: target_ta_name,
        }) = event_wrapper.get::<InputForFeeding>()
        {
            let db_opt = dialog_box_query
                .iter_mut()
                .find(|x| x.0.name == target_db_name);
            let ta_opt = text_area_query.iter().find(|x| x.1.name == target_ta_name);
            if let (Some((_, mut dbp)), Some((ta_entity, _, fs))) = (db_opt, ta_opt) {
                *dbp = DialogBoxPhase::Typing;
                for (l_entity, l_parent) in &mut line_query {
                    if l_parent.get() == ta_entity {
                        commands.entity(l_entity).insert(*fs);
                        *dbp = DialogBoxPhase::WaitingAction;
                    }
                    start_feeding_event.send(StartFeedingEvent);
                }
                for (ic_entity, mut ic_vis) in &mut icon_query {
                    *ic_vis = Visibility::Hidden;
                    commands.entity(ic_entity).remove::<TypingTimer>();
                }
            }
        }
    }
}

pub fn trigger_feeding_by_time(
    mut commands: Commands,
    mut dialog_box_query: Query<&mut DialogBoxPhase>,
    mut text_area_query: Query<(Entity, &FeedingStyle, &mut WaitFeedingTrigger), With<Current>>,
    mut line_query: Query<Entity, With<MessageTextLine>>,
    parent_query: Query<&Parent>,
    mut start_feeding_event: EventWriter<StartFeedingEvent>,
    time: Res<Time>,
) {
    for mut dbp in &mut dialog_box_query {
        if text_area_query.iter().len() > 0 {
            *dbp = DialogBoxPhase::Typing;
        }
        for (tb_entity, fs, mut wft) in &mut text_area_query {
            if wft.timer.tick(time.delta()).finished() {
                for l_entity in &mut line_query {
                    if parent_query.get(l_entity).ok().map(|x| x.get()) == Some(tb_entity) {
                        commands.entity(l_entity).insert(*fs);
                        *dbp = DialogBoxPhase::WaitingAction;
                    }
                }
                commands.entity(tb_entity).remove::<WaitFeedingTrigger>();
                start_feeding_event.send(StartFeedingEvent);
            }
        }
    }
}

pub fn start_feeding(
    mut commands: Commands,
    mut window_query: Query<(Entity, &DialogBox, &mut DialogBoxPhase, &WaitBrakerStyle)>,
    text_box_query: Query<(Entity, &TextArea, &GlobalTransform, &Sprite, &Parent)>,
    parent_query: Query<&Parent>,
    line_query: Query<(Entity, &FeedingStyle), With<MessageTextLine>>,
    start_feeding_event: EventReader<StartFeedingEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    if start_feeding_event.is_empty() {
        return;
    };
    for (w_entity, db, mut ws, wbs) in &mut window_query {
        if *ws != DialogBoxPhase::WaitingAction {
            continue;
        }
        let target_lines = line_query
            .iter()
            .filter(|q| parent_query.iter_ancestors(q.0).any(|e| e == w_entity))
            .collect::<Vec<(Entity, &FeedingStyle)>>();
        if target_lines.iter().len() > 0 {
            *ws = DialogBoxPhase::Typing;
        }
        for (l_entity, fs) in target_lines.iter() {
            match fs {
                FeedingStyle::Scroll {
                    size: fs_size,
                    sec: fs_sec,
                } => {
                    let line_size = line_query.iter().count();
                    let line_count = if *fs_size == 0 || line_size < *fs_size {
                        0
                    } else {
                        line_size - *fs_size
                    };
                    commands.entity(*l_entity).insert(ScrollFeed {
                        line_per_sec: target_lines.len() as f32 / *fs_sec,
                        count: line_count,
                    });
                    *ws = DialogBoxPhase::Feeding;
                }
                FeedingStyle::Rid => {
                    commands.entity(*l_entity).despawn_recursive();
                }
            };
        }
        for (tb_entity, ta, tb_tf, tb_sp, tb_parent) in &text_box_query {
            if tb_parent.get() == w_entity {
                if let WaitBrakerStyle::Input {
                    is_all_range_area: is_all_range,
                    ..
                } = wbs
                {
                    let empty = "".to_string();
                    let wig = if *is_all_range {
                        make_wig_for_skip_all_range(&db.name, &ta.name, &empty, &type_registry)
                    } else {
                        make_wig_for_skip(&db.name, &ta.name, tb_tf, tb_sp, &empty, &type_registry)
                    };
                    commands.entity(tb_entity).insert(wig);
                }
            }
        }
    }
}

pub fn scroll_lines(
    mut commands: Commands,
    mut window_query: Query<(Entity, &mut DialogBoxPhase)>,
    mut line_query: Query<(Entity, &mut Transform, &Sprite, &mut ScrollFeed)>,
    parent_query: Query<&Parent>,
    time: Res<Time>,
) {
    for (w_entity, mut ws) in &mut window_query {
        if *ws == DialogBoxPhase::Feeding {
            let mut target_lines = line_query
                .iter_mut()
                .filter(|q| parent_query.iter_ancestors(q.0).any(|e| e == w_entity))
                .collect::<Vec<(Entity, Mut<Transform>, &Sprite, Mut<ScrollFeed>)>>();
            target_lines.sort_by(|a, b| a.1.translation.y.partial_cmp(&b.1.translation.y).unwrap());
            let targets_size = target_lines.len();
            if targets_size <= target_lines.first().map(|l| l.3.count).unwrap_or_default() {
                *ws = DialogBoxPhase::Typing;
                for (l_entity, _, _, _) in target_lines.iter() {
                    commands.entity(*l_entity).remove::<ScrollFeed>();
                }
            } else {
                let height = target_lines
                    .first()
                    .and_then(|x| x.2.custom_size.map(|s| s.y))
                    .unwrap_or_default();
                for (l_entity, ref mut tf, _, ref mut sf) in target_lines.iter_mut() {
                    tf.translation.y += height * sf.line_per_sec * time.delta_seconds();
                    if tf.translation.y >= -height {
                        tf.scale.y -= time.delta_seconds() * sf.line_per_sec;
                        if tf.scale.y <= 0. {
                            commands.entity(*l_entity).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}
