use super::super::window_controller::waiting::*;
use super::super::*;

#[derive(Event)]
pub(in crate::writing) struct FeedWaitingEvent {
    pub target_box_name: String,
    pub wait_sec: f32,
    // pub last_pos: Vec2,
}

#[derive(Event)]
pub(in crate::writing) struct StartFeedingEvent {
    pub target_box_name: String,
    pub target_area_name: String,
}

#[derive(Component)]
pub(in crate::writing) struct WaitFeedingTrigger {
    pub timer: Timer,
}

#[derive(Component)]
pub(in crate::writing) struct ScrollFeed {
    pub line_per_sec: f32,
    pub count: usize,
}

// 改ページを読み込んだ時点でTextAreaにInputForFeedingを発行するWaitInputGoをセットします。
// InputForFeedingはWaitInputGo内のInputForSkipping内にあり、Skip後の動作として登録されます。
// 発行される時点ではtypeが終わっていない可能性が高いからです。
#[allow(clippy::type_complexity)]
pub(in crate::writing) fn setup_feed_starter(
    mut commands: Commands,
    writing_query: Query<(Entity, &WaitBrakerStyle, &DialogBox)>,
    text_box_query: Query<(Entity, &TextArea, &ChildOf, &GlobalTransform, &Sprite), With<Current>>,
    w_icon_query: Query<(Entity, &WaitingIcon)>,
    mut waitting_event: EventReader<FeedWaitingEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event in waitting_event.read() {
        for (db_entity, wbs, DialogBox { name: db_name }) in &writing_query {
            for (ta_entity, ta, parent, tb_tf, tb_sp) in &text_box_query {
                if event.target_box_name != *db_name || db_entity != parent.parent() {
                    break;
                }
                match wbs {
                    WaitBrakerStyle::Auto { wait_sec: break_ws } => {
                        commands.entity(ta_entity).insert(WaitFeedingTrigger {
                            timer: Timer::from_seconds(event.wait_sec + break_ws, TimerMode::Once),
                        });
                    }
                    WaitBrakerStyle::Input {
                        is_all_range_area: is_all_range,
                        ..
                    } => {
                        let icon_opt = w_icon_query.iter().find(|x| {
                            *db_name == x.1.target_box_name
                                && x.1.wait_for.contains(&WaitTarget::Feeding)
                        });
                        if let Some((ic_entity, _)) = icon_opt {
                            let tt = TypingTimer {
                                timer: Timer::from_seconds(event.wait_sec, TimerMode::Once),
                            };
                            commands.entity(ic_entity).insert(tt);
                            commands.entity(ic_entity).insert(ChildOf(ta_entity));
                        }
						let iff = InputForFeeding {
                                writing_name: db_name.clone(),
                                text_area_name: ta.name.clone(),
                            },
                        let ron_iff = write_ron(&type_registry, iff).unwrap_or_default();
                        let wig = if *is_all_range {
                            make_wig_for_skip_all_range(db_name, &ta.name, &ron_iff, &type_registry)
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
                        commands.entity(ta_entity).insert(wig);
                        commands.entity(ta_entity).insert(Selected);
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub(in crate::writing) fn trigger_feeding_by_event(
    mut commands: Commands,
    mut line_query: Query<(Entity, &ChildOf), With<MessageTextLine>>,
    mut writing_query: Query<(&DialogBox, &mut DialogBoxPhase)>,
    text_area_query: Query<(Entity, &TextArea, &FeedingStyle), With<Current>>,
    mut icon_query: Query<(Entity, &mut Visibility), With<WaitingIcon>>,
    mut start_feeding_event: EventWriter<StartFeedingEvent>,
    mut events: EventReader<BdsEvent>,
) {
    for event_wrapper in events.read() {
        if let Some(InputForFeeding {
            writing_name: target_db_name,
            text_area_name: target_ta_name,
        }) = event_wrapper.get::<InputForFeeding>()
        {
            let db_opt = writing_query
                .iter_mut()
                .find(|x| x.0.name == target_db_name);
            let ta_opt = text_area_query.iter().find(|x| x.1.name == target_ta_name);
            if let (Some((db, mut dbp)), Some((ta_entity, ta, fs))) = (db_opt, ta_opt) {
                *dbp = DialogBoxPhase::Typing;
                for (l_entity, l_parent) in &mut line_query {
                    if l_parent.parent() == ta_entity {
                        commands.entity(l_entity).insert(*fs);
                        *dbp = DialogBoxPhase::WaitingAction;
                    }
                    start_feeding_event.write(StartFeedingEvent {
                        target_box_name: db.name.clone(),
                        target_area_name: ta.name.clone(),
                    });
                }
                for (ic_entity, mut ic_vis) in &mut icon_query {
                    *ic_vis = Visibility::Hidden;
                    commands.entity(ic_entity).remove::<TypingStyle>();
                    commands.entity(ic_entity).remove::<TypingTimer>();
                }
            }
        }
    }
}

pub(in crate::writing) fn trigger_feeding_by_time(
    mut commands: Commands,
    mut writing_query: Query<(&DialogBox, &mut DialogBoxPhase)>,
    mut text_area_query: Query<
        (Entity, &TextArea, &FeedingStyle, &mut WaitFeedingTrigger),
        With<Current>,
    >,
    mut line_query: Query<Entity, With<MessageTextLine>>,
    parent_query: Query<&ChildOf>,
    mut start_feeding_event: EventWriter<StartFeedingEvent>,
    time: Res<Time>,
) {
    for (db, mut dbp) in &mut writing_query {
        if text_area_query.iter().len() > 0 {
            *dbp = DialogBoxPhase::Typing;
        }
        for (ta_entity, ta, fs, mut wft) in &mut text_area_query {
            if wft.timer.tick(time.delta()).finished() {
                for l_entity in &mut line_query {
                    if parent_query.get(l_entity).ok().map(|x| x.parent()) == Some(ta_entity) {
                        commands.entity(l_entity).insert(*fs);
                        *dbp = DialogBoxPhase::WaitingAction;
                    }
                }
                commands.entity(ta_entity).remove::<WaitFeedingTrigger>();
                start_feeding_event.write(StartFeedingEvent {
                    target_box_name: db.name.clone(),
                    target_area_name: ta.name.clone(),
                });
            }
        }
    }
}

pub(in crate::writing) fn start_feeding(
    mut commands: Commands,
    mut window_query: Query<(&DialogBox, &mut DialogBoxPhase, &WaitBrakerStyle)>,
    text_box_query: Query<(Entity, &TextArea, &GlobalTransform, &Sprite)>,
    line_query: Query<(Entity, &FeedingStyle, &ChildOf), With<MessageTextLine>>,
    mut start_feeding_event: EventReader<StartFeedingEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for sf in start_feeding_event.read() {
        for (db, mut ws, wbs) in &mut window_query {
            if db.name != sf.target_box_name || *ws != DialogBoxPhase::WaitingAction {
                continue;
            }
            for (ta_entity, ta, tb_tf, tb_sp) in &text_box_query {
                if ta.name != sf.target_area_name {
                    continue;
                }
                let target_lines = line_query
                    .iter()
                    .filter(|q| q.2.parent() == ta_entity)
                    .map(|q| (q.0, q.1))
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
                            commands.entity(*l_entity).despawn();
                        }
                    };
                }
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
                    commands.entity(ta_entity).insert(wig);
                }
            }
        }
    }
}

pub(in crate::writing) fn scroll_lines(
    mut commands: Commands,
    mut window_query: Query<(Entity, &mut DialogBoxPhase)>,
    mut line_query: Query<(Entity, &mut Transform, &Sprite, &mut ScrollFeed)>,
    parent_query: Query<&ChildOf>,
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
                    tf.translation.y += height * sf.line_per_sec * time.delta_secs();
                    if tf.translation.y >= -height {
                        tf.scale.y -= time.delta_secs() * sf.line_per_sec;
                        if tf.scale.y <= 0. {
                            commands.entity(*l_entity).despawn();
                        }
                    }
                }
            }
        }
    }
}
