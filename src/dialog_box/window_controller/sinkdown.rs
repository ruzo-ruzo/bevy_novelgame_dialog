use super::*;
use crate::dialog_box::*;

#[derive(Component, Debug)]
pub(in crate::dialog_box) struct ScalingDown {
    pub sub_per_sec: f32,
}

#[derive(Component)]
pub(in crate::dialog_box) struct Despawning;

#[derive(Component, Debug)]
pub(in crate::dialog_box) struct WaitSinkingTrigger {
    pub sink_type: SinkDownType,
    pub timer: Timer,
}

#[derive(Reflect, Default, Event)]
pub(in crate::dialog_box) struct GoSinking {
    pub dialog_box_name: String,
    pub sink_type: SinkDownType,
}

#[allow(clippy::type_complexity)]
pub(in crate::dialog_box) fn setup_window_sink(
    mut commands: Commands,
    text_query: Query<(Entity, &TypingTimer), (With<Current>, With<MessageTextChar>)>,
    text_box_query: Query<(Entity, &TextArea, &GlobalTransform, &Sprite), With<Current>>,
    mut db_query: Query<(Entity, &DialogBox, &mut DialogBoxPhase, &WaitBrakerStyle), With<Current>>,
    parents: Query<&Parent>,
    mut events: EventReader<BdsEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in events.read() {
        if let Some(SinkDownWindow { sink_type: sdt }) = event_wrapper.get::<SinkDownWindow>() {
            for (mw_entity, db, mut ws, wbs) in &mut db_query {
                match wbs {
                    WaitBrakerStyle::Auto { wait_sec: base_sec } => {
                        let count: f32 = text_query
                            .iter()
                            .filter(|(tx_entity, _)| {
                                parents.iter_ancestors(*tx_entity).any(|x| x == mw_entity)
                            })
                            .map(|(_, tt)| tt.timer.remaining_secs())
                            .sum();
                        commands.entity(mw_entity).insert(WaitSinkingTrigger {
                            sink_type: sdt,
                            timer: Timer::from_seconds(base_sec + count, TimerMode::Once),
                        });
                    }
                    WaitBrakerStyle::Input { .. } => {
                        if let Some((target_tb, ta, tb_tf, tb_sp)) =
                            text_box_query.iter().find(|(tb_e, ..)| {
                                parents.iter_ancestors(*tb_e).any(|pa_e| pa_e == mw_entity)
                            })
                        {
                            let gs_ron = write_ron(
                                &type_registry,
                                GoSinking {
                                    dialog_box_name: db.name.clone(),
                                    sink_type: sdt,
                                },
                            );
                            let wig = make_wig_for_skip(
                                &db.name,
                                &ta.name,
                                tb_tf,
                                tb_sp,
                                &gs_ron.unwrap_or_default(),
                                &type_registry,
                            );
                            commands.entity(target_tb).insert(wig);
                        }
                    }
                }
                *ws = DialogBoxPhase::WaitingAction
            }
        }
    }
}

pub(in crate::dialog_box) fn trigger_window_sink_by_event(
    mut bds_reader: EventReader<BdsEvent>,
    mut gs_writer: EventWriter<GoSinking>,
) {
    for event_wrapper in bds_reader.read() {
        if let Some(gs @ GoSinking { .. }) = event_wrapper.get::<GoSinking>() {
            gs_writer.send(gs);
        }
    }
}

pub(in crate::dialog_box) fn trigger_window_sink_by_time(
    mut commands: Commands,
    mut db_query: Query<(Entity, &DialogBox, &mut WaitSinkingTrigger)>,
    time: Res<Time>,
    mut events: EventWriter<GoSinking>,
) {
    for (entity, db, mut wst) in &mut db_query {
        if wst.timer.tick(time.delta()).finished() {
            events.send(GoSinking {
                dialog_box_name: db.name.clone(),
                sink_type: wst.sink_type,
            });
            commands.entity(entity).remove::<WaitSinkingTrigger>();
        }
    }
}

pub(in crate::dialog_box) fn start_window_sink(
    mut commands: Commands,
    mut db_query: Query<(Entity, &DialogBox, &mut DialogBoxPhase)>,
    mut events: EventReader<GoSinking>,
) {
    for GoSinking {
        dialog_box_name: db_name,
        sink_type: st,
    } in &mut events.read()
    {
        for (entity, db, mut ws) in &mut db_query {
            if db.name == *db_name {
                match st {
                    SinkDownType::Scale { sec: s } => {
                        commands.entity(entity).insert(ScalingDown {
                            sub_per_sec: 1.0 / s,
                        });
                        *ws = DialogBoxPhase::SinkingDown;
                    }
                    SinkDownType::Fix => {
                        *ws = DialogBoxPhase::Fixed;
                        commands.entity(entity).remove::<Current>();
                    }
                }
            }
        }
    }
}

// Todo: テキストが素っ頓狂な方向へ飛んでくの直したい
pub(in crate::dialog_box) fn scaling_down(
    mut commands: Commands,
    mut db_query: Query<(Entity, &mut Transform, &ScalingDown)>,
    time: Res<Time>,
) {
    for (entity, mut tf, ScalingDown { sub_per_sec: aps }) in &mut db_query {
        if tf.scale.x <= 0.0 {
            tf.scale = Vec3::new(0., 0., 0.);
            commands
                .entity(entity)
                .remove::<ScalingDown>()
                .insert(Despawning);
        } else {
            tf.scale.x -= time.delta_seconds() * aps;
            tf.scale.y -= time.delta_seconds() * aps;
        };
    }
}

// waiting iconだけ残すんじゃなくてlineとかだけ消す？
// （line以外の候補洗わないとだが。SelectedとかCurrentとか）
pub(in crate::dialog_box) fn despawn_dialog_box(
    mut commands: Commands,
    db_query: Query<(Entity, &DialogBox), With<Despawning>>,
    w_icon_query: Query<&WaitingIcon>,
    ta_query: Query<Entity, With<TextArea>>,
    ch_query: Query<&Children>,
    instant_query: Query<&Instant>,
    mut event: EventWriter<FinisClosingBox>,
) {
    for (db_entity, db) in &db_query {
        if let Ok(tb_children) = ch_query.get(db_entity) {
            for tb_childe in tb_children {
                if let Ok(ta_entity) = ta_query.get(*tb_childe) {
                    if let Ok(ta_children) = ch_query.get(ta_entity) {
                        for ta_childe in ta_children {
                            if w_icon_query.get(*ta_childe).is_ok() {
                                commands.entity(ta_entity).remove_children(&[*ta_childe]);
                            } else {
                                commands.entity(*ta_childe).despawn_recursive();
                            }
                        }
                    }
                    commands.entity(ta_entity).despawn();
                }
            }
        }
        commands
            .entity(db_entity)
            .remove::<DialogBoxBundle>()
            .remove::<Current>()
            .remove::<Despawning>();
        if instant_query.get(db_entity).is_ok() {
            commands.entity(db_entity).despawn();
        }
        event.send(FinisClosingBox {
            dialog_box_name: db.name.clone(),
        });
    }
}

#[allow(clippy::type_complexity)]
pub(in crate::dialog_box) fn remove_pending(
    mut commands: Commands,
    mut pending_query: Query<(Entity, &mut DialogBoxPhase), (With<DialogBox>, With<Pending>)>,
    current_db_query: Query<&Current, (With<DialogBox>, Without<Pending>)>,
    children_query: Query<&Children>,
) {
    if current_db_query.iter().next().is_none() {
        if let Ok((db_entity, mut dbp)) = pending_query.get_single_mut() {
            commands.entity(db_entity).remove::<Pending>();
            commands.entity(db_entity).insert(Current);
            if let Ok(children) = children_query.get(db_entity) {
                for childe in children {
                    commands.entity(*childe).remove::<Pending>();
                }
            }
            if *dbp == DialogBoxPhase::WaitToType {
                *dbp = DialogBoxPhase::Typing;
            }
        }
    }
}
