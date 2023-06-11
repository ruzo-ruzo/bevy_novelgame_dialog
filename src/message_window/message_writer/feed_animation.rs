use bevy::prelude::*;

use super::super::*;

#[derive(Event, Debug)]
pub struct FeedWaitingEvent {
    pub target_window: Entity,
    pub wait_sec: f32,
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
    window_query: Query<(Entity, &WaitBrakerStyle)>,
    text_box_query: Query<(Entity, &Parent), (With<Current>, With<TextBox>)>,
    mut waitting_event: EventReader<FeedWaitingEvent>,
) {
    for event in waitting_event.iter() {
        for (w_entity, wbs) in &window_query {
            for (tb_entity, parent) in &text_box_query {
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
                    }
                }
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
