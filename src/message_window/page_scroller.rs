use super::*;
use crate::utility::*;
use bevy::{ecs::component::Component, time::Timer};

#[derive(Component, Debug)]
pub struct ScrollUp {
    row_per_sec: f32,
}

#[derive(Component, Debug)]
pub struct RemoveUp {
    row_per_sec: f32,
    end_timer: Timer,
}

#[derive(Component, Debug)]
pub struct ScrollWaiting {
    wait_reading: Timer,
}

#[derive(Default)]
pub struct LineFeedEvent;

#[derive(Default)]
pub struct PageScrollEvent;

#[derive(Default)]
pub struct ScrollWaitingEvent;

pub fn trigger_page_scroll(
    mut commands: Commands,
    config: ResMut<MessageWindowConfig>,
    mut ps_event: EventReader<PageScrollEvent>,
    mut lf_event: EventWriter<LineFeedEvent>,
    mut waiting_timer: Query<(Entity, &mut ScrollWaiting)>,
    time: Res<Time>,
) {
    if !ps_event.is_empty() {
        commands.spawn(ScrollWaiting {
            wait_reading: Timer::from_seconds(config.wait_reading_time, TimerMode::Once),
        });
        ps_event.clear();
    } else if let Ok((entity, mut sw)) = waiting_timer.get_single_mut() {
        if sw.wait_reading.tick(time.delta()).finished() {
            lf_event.send_default();
            commands.entity(entity).despawn();
        }
    }
}

pub fn start_page_scroll(
    mut commands: Commands,
    mut config: ResMut<MessageWindowConfig>,
    window_children_query: Query<&Children, With<CurrentMessageWindow>>,
    mta_children_query: Query<&Children, With<MessageTextArea>>,
    lines_query: Query<(Entity, &mut MessageTextLine)>,
    mut lf_event: EventReader<LineFeedEvent>,
) {
    if !lf_event.is_empty() {
        let current_mta_entities = get_child_entities(window_children_query.iter());
        let current_lines_entities =
            down_children_entity(current_mta_entities.iter(), &mta_children_query);
        let mut current_lines = lines_query
            .iter_many(current_lines_entities)
            .collect::<Vec<(Entity, &MessageTextLine)>>();
        current_lines.sort_by_key(|x| x.1.row_index);
        let mut iter = current_lines.iter();
        let first_line = iter.next();
        config.wait_ps_count = match config.wait_ps_count {
            Some(1) => {
                config.state = MessageWindowState::Writing;
                None
            }
            Some(u) => Some(u - 1),
            None if config.scroll_size == 0 => Some(current_lines.len()),
            None => Some(config.scroll_size),
        };
        if config.wait_ps_count.is_none() {
            return;
        }
        // ↓現状Scrollingステータスにする意味はないが一応
        config.state = MessageWindowState::Scrolling;
        if let Some(l) = first_line {
            commands.entity(l.0).insert(RemoveUp {
                row_per_sec: config.scroll_speed,
                end_timer: Timer::from_seconds(1.0 / config.scroll_speed, TimerMode::Once),
            });
        };
        for l in iter {
            commands.entity(l.0).insert(ScrollUp {
                row_per_sec: config.scroll_speed,
            });
        }
        lf_event.clear();
    };
}

pub fn scroll_lines(
    mut lines_query: Query<(&ScrollUp, &mut Transform, &MessageTextLine)>,
    time: Res<Time>,
) {
    for (s, mut t, l) in &mut lines_query {
        t.translation.y -= l.max_char_height * s.row_per_sec * time.delta_seconds();
        if t.translation.y > 0. {
            t.translation.y = 0.0;
        }
    }
}

//start_page_scrollより先に呼んでしまうと、RemoveUpが消えないうちに再発行してしまうせいで落ちる
pub fn remove_lines(
    mut commands: Commands,
    mut lines_query: Query<(Entity, &mut RemoveUp, &mut Transform)>,
    mut scrolling_query: Query<Entity, With<ScrollUp>>,
    mut ps_event: EventWriter<LineFeedEvent>,
    time: Res<Time>,
) {
    for (e, mut s, mut t) in &mut lines_query {
        t.scale.y -= time.delta_seconds() / s.row_per_sec;
        if t.scale.y < 0. {
            t.scale.y = 0.0;
        }
        if s.end_timer.tick(time.delta()).finished() {
            for se in &mut scrolling_query {
                commands.entity(se).remove::<ScrollUp>();
            }
            commands.entity(e).despawn_recursive();
            ps_event.send_default();
        }
    }
}
