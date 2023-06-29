use super::super::*;
use super::skip_typing::*;
use bevy::prelude::*;

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

#[derive(Reflect, Default, Debug)]
pub struct InputForSkipping {
    pub next_event_ron: String,
    pub target_text_box: Option<Entity>,
}

#[derive(Component, Debug)]
pub struct ScrollFeed {
    pub line_per_sec: f32,
    pub count: usize,
}

#[derive(Component, Debug)]
pub struct WatingIcon;

#[allow(clippy::type_complexity)]
pub fn setup_feed_starter(
    mut commands: Commands,
    window_query: Query<(Entity, &WaitBrakerStyle)>,
    text_box_query: Query<
        (Entity, &Parent, &TypeTextConfig, &GlobalTransform, &Sprite),
        With<TextBox>,
    >,
    mut icon_query: Query<&mut Transform>,
    selected_query: Query<Entity, With<Selected>>,
    mut waitting_event: EventReader<FeedWaitingEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event in waitting_event.iter() {
        for (w_entity, wbs) in &window_query {
            for (tb_entity, parent, config, tb_tf, tb_sp) in &text_box_query {
                if event.target_window == w_entity && w_entity == parent.get() {
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
                            icon_entity: icon_opt,
                            is_icon_moving_to_last: move_flag,
                        } => {
                            if let Some(ic_entity) = icon_opt {
                                if let Ok(mut ic_tf) = icon_query.get_mut(*ic_entity) {
                                    if *move_flag {
                                        ic_tf.translation = Vec3::new(
                                            event.last_pos.x + config.text_style.font_size,
                                            event.last_pos.y,
                                            1.,
                                        );
                                    }
                                    let tt = TypingTimer {
                                        timer: Timer::from_seconds(event.wait_sec, TimerMode::Once),
                                    };
                                    commands.entity(*ic_entity).insert((
                                        tt,
                                        WatingIcon,
                                        WritingStyle::Put,
                                    ));
                                    commands.entity(*ic_entity).set_parent(tb_entity);
                                }
                            }
                            commands.entity(tb_entity).insert(make_wig_for_textbox(
                                tb_entity,
                                tb_tf,
                                tb_sp,
                                &type_registry,
                            ));
                            for s_entity in &selected_query {
                                commands.entity(s_entity).remove::<Selected>();
                            }
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
    text_box_query: Query<&FeedingStyle>,
    mut icon_query: Query<(Entity, &mut Visibility), (With<WatingIcon>, Without<MessageTextChar>)>,
    mut start_feeding_event: EventWriter<StartFeedingEvent>,
    mut events: EventReader<BMSEvent>,
) {
    for event_wrapper in events.iter() {
        if let Some(InputForFeeding {
            target_text_box: Some(tb_entity),
        }) = event_wrapper.get_opt::<InputForFeeding>()
        {
            for (l_entity, l_parent) in &mut line_query {
                if l_parent.get() == tb_entity {
                    if let Ok(fs) = text_box_query.get(tb_entity) {
                        commands.entity(l_entity).insert(*fs);
                    }
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

pub fn trigger_feeding_by_time(
    mut commands: Commands,
    mut text_box_query: Query<(Entity, &FeedingStyle, &mut WaitFeedingTrigger)>,
    mut line_query: Query<(Entity, &Parent), With<MessageTextLine>>,
    mut start_feeding_event: EventWriter<StartFeedingEvent>,
    time: Res<Time>,
) {
    for (tb_entity, fs, mut wft) in &mut text_box_query {
        if wft.timer.tick(time.delta()).finished() {
            for (l_entity, parent) in &mut line_query {
                if parent.get() == tb_entity {
                    commands.entity(l_entity).insert(*fs);
                }
            }
            commands.entity(tb_entity).remove::<WaitFeedingTrigger>();
            start_feeding_event.send(StartFeedingEvent);
        }
    }
}

pub fn start_feeding(
    mut commands: Commands,
    mut window_query: Query<(Entity, &mut WindowState)>,
    parent_query: Query<&Parent>,
    line_query: Query<(Entity, &FeedingStyle), With<MessageTextLine>>,
    start_feeding_event: EventReader<StartFeedingEvent>,
) {
    if start_feeding_event.is_empty() {
        return;
    };
    for (w_entity, mut ws) in &mut window_query {
        let target_lines = line_query
            .iter()
            .filter(|q| parent_query.iter_ancestors(q.0).any(|e| e == w_entity))
            .collect::<Vec<(Entity, &FeedingStyle)>>();
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
                        line_per_sec: *fs_sec,
                        count: line_count,
                    })
                }
            };
            *ws = WindowState::Feeding;
        }
    }
}

pub fn scroll_lines(
    mut commands: Commands,
    mut window_query: Query<(Entity, &mut WindowState)>,
    mut line_query: Query<(Entity, &mut Transform, &Sprite, &mut ScrollFeed)>,
    parent_query: Query<&Parent>,
    time: Res<Time>,
) {
    for (w_entity, mut ws) in &mut window_query {
        if *ws == WindowState::Feeding {
            let mut target_lines = line_query
                .iter_mut()
                .filter(|q| parent_query.iter_ancestors(q.0).any(|e| e == w_entity))
                .collect::<Vec<(Entity, Mut<Transform>, &Sprite, Mut<ScrollFeed>)>>();
            target_lines.sort_by(|a, b| a.1.translation.y.partial_cmp(&b.1.translation.y).unwrap());
            let targets_size = target_lines.len();
            if targets_size == target_lines.first().map(|l| l.3.count).unwrap_or_default() {
                *ws = WindowState::Typing;
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
