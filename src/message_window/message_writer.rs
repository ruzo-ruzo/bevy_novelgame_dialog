use bevy::{prelude::*, sprite::Anchor, text::Font, time::Timer};

use super::utility::*;
use super::*;

#[derive(Component, Debug, PartialEq)]
pub enum MessageWindowState {
    NoWindow,
    Waiting,
    Writing,
    Scrolling,
}

impl Default for MessageWindowState {
    fn default() -> MessageWindowState {
        Self::NoWindow
    }
}

#[derive(Component, Debug)]
pub struct MessageWindow;

#[derive(Component, Debug)]
pub struct CurrentMessageWindow;

#[derive(Component, Debug)]
pub struct MessageTextArea {
    pub area_size: Vec2,
}

#[derive(Component, Debug)]
pub struct MessageTextLine {
    row_index: usize,
    max_char_height: f32,
}

#[derive(Component, Debug)]
pub struct MessageTextChar {
    line_index: usize,
}

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

pub fn update_message(
    commands: Commands,
    sub_commands: Commands,
    fonts: Res<Assets<Font>>,
    mut config: ResMut<MessageWindowConfig>,
    window_children_query: Query<&Children, With<CurrentMessageWindow>>,
    mta_children_query: Query<&Children, With<MessageTextArea>>,
    lines_children_query: Query<&Children, With<MessageTextLine>>,
    mta_query: Query<&MessageTextArea>,
    lines_query: Query<(Entity, &MessageTextLine, &Transform)>,
    chars_query: Query<(&MessageTextChar, &Transform, &Text)>,
    loded_text: ResMut<LoadedText>,
    time: Res<Time>,
    mut timer_config: ResMut<MessageDisplayConfig>,
    mut ps_event: EventWriter<PageScrollEvent>,
    mut pending: Local<Option<char>>,
) {
    timer_config.char_add_timer.tick(time.delta());
    if config.state == MessageWindowState::Writing && timer_config.char_add_timer.finished() {
        let mut new_text = pending.clone();
        if new_text.is_none() {
            new_text = get_next_text(loded_text)
        };
        if new_text.is_none() {
            return;
        }
        let new_char = new_text.unwrap();
        match new_char {
            '\t' => {
                ps_event.send_default();
                config.state = MessageWindowState::Waiting
            }
            _ => {
                let is_new_char_added = add_character(
                    commands,
                    sub_commands,
                    fonts,
                    config.reborrow(),
                    window_children_query,
                    mta_children_query,
                    lines_children_query,
                    mta_query,
                    lines_query,
                    chars_query,
                    new_char,
                );
                if is_new_char_added {
                    *pending = None;
                } else {
                    ps_event.send_default();
                    config.state = MessageWindowState::Waiting;
                    *pending = Some(new_char);
                }
            }
        }
    }
}

fn add_character(
    mut commands: Commands,
    sub_commands: Commands,
    fonts: Res<Assets<Font>>,
    mut config: Mut<'_, MessageWindowConfig>,
    window_children_query: Query<&Children, With<CurrentMessageWindow>>,
    mta_children_query: Query<&Children, With<MessageTextArea>>,
    lines_children_query: Query<&Children, With<MessageTextLine>>,
    mta_query: Query<&MessageTextArea>,
    lines_query: Query<(Entity, &MessageTextLine, &Transform)>,
    chars_query: Query<(&MessageTextChar, &Transform, &Text)>,
    new_char: char,
) -> bool {
    let current_mta_entities = get_child_entities(window_children_query.iter());
    let current_lines_entities =
        down_children_entity(current_mta_entities.iter(), &mta_children_query);
    let current_lines = lines_query.iter_many(current_lines_entities);
    let last_line_entity = current_lines.max_by_key(|x| x.1.row_index).map(|x| x.0);

    let last_text_entities = last_line_entity
        .map(|x| entity_get_children(&x, &lines_children_query))
        .unwrap_or_default();
    let last_text = chars_query.iter_many(last_text_entities.iter());
    let last_char = last_text.max_by_key(|x| x.0.line_index);
    let last_char_ts = last_char.and_then(|c| c.2.sections.first());
    let left_end = last_char.map(|c| c.1.translation.x).unwrap_or(0.)
        + last_char_ts.map(|s| s.style.font_size).unwrap_or(0.)
        + config.current_text_style.font_size;
    let area_width = current_mta_entities
        .first()
        .and_then(|e| mta_query.get(*e).ok())
        .map(|a| a.area_size.x);
    let is_overflow = left_end > area_width.unwrap_or(0.) || last_line_entity.is_none();
    let is_linefeed = new_char == '\n';

    let last_char_x = if is_overflow || is_linefeed {
        0.
    } else {
        last_char
            .map(|c| c.1.translation.x + config.current_text_style.font_size)
            .unwrap_or(0.)
    };
    let next_line_index = if is_overflow || is_linefeed {
        1
    } else {
        last_char.map(|c| c.0.line_index).unwrap_or(0) + 1
    };
    let text_style = TextStyle {
        font: choice_font(&config.font_handles, new_char, fonts).unwrap_or_default(),
        ..config.current_text_style
    };

    let next_line = if is_overflow || is_linefeed {
        add_empty_line(
            sub_commands,
            config.reborrow(),
            window_children_query,
            mta_children_query,
            mta_query,
            lines_query,
        )
    } else {
        last_line_entity
    };

    if next_line.is_none() {
        return false;
    };

    if !is_linefeed {
        let new_message_char = commands
            .spawn((
                Text2dBundle {
                    text: Text::from_section(new_char.to_string(), text_style),
                    transform: Transform::from_translation(Vec3::new(last_char_x, 0., 1.)),
                    text_anchor: Anchor::TopLeft,
                    ..default()
                },
                config.layers,
                MessageTextChar {
                    line_index: next_line_index,
                },
            ))
            .id();
        if let Some(e) = next_line {
            commands.entity(e).add_child(new_message_char);
        }
    }
    true
}

fn add_empty_line(
    mut commands: Commands,
    config: Mut<'_, MessageWindowConfig>,
    window_query: Query<&Children, With<CurrentMessageWindow>>,
    mta_children_query: Query<&Children, With<MessageTextArea>>,
    mta_query: Query<&MessageTextArea>,
    lines_query: Query<(Entity, &MessageTextLine, &Transform)>,
) -> Option<Entity> {
    let current_mta_entities = get_child_entities(window_query.iter());
    let current_lines_entities =
        down_children_entity(current_mta_entities.iter(), &mta_children_query);
    let lines = lines_query.iter_many(current_lines_entities);
    let last_line = lines.max_by_key(|x| x.1.row_index);

    let next_lines_origin_height = last_line
        .map(|x| x.2.translation.y + x.1.max_char_height)
        .unwrap_or(0.);
    let next_index = last_line.map(|x| x.1.row_index).unwrap_or(0) + 1;

    let area_height = current_mta_entities
        .first()
        .and_then(|e| mta_query.get(*e).ok())
        .map(|a| a.area_size.y);
    if -next_lines_origin_height + config.current_text_style.font_size > area_height.unwrap_or(0.) {
        return None;
    };

    let mtl = commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(0., next_lines_origin_height, 0.)),
                ..default()
            },
            config.layers,
            MessageTextLine {
                row_index: next_index,
                max_char_height: -config.current_text_style.font_size,
            },
        ))
        .id();
    if let Some(mta) = current_mta_entities.get(0) {
        commands.entity(*mta).add_child(mtl);
    }
    Some(mtl)
}

pub fn update_line_height(
    window_children_query: Query<&Children, With<CurrentMessageWindow>>,
    mta_children_query: Query<&Children, With<MessageTextArea>>,
    lines_children_query: Query<&Children, With<MessageTextLine>>,
    char_query: Query<&Text, With<MessageTextChar>>,
    mut lines_query: Query<&mut MessageTextLine>,
) {
    let current_mta_entities = get_child_entities(window_children_query.iter());
    let current_lines_entities =
        down_children_entity(current_mta_entities.iter(), &mta_children_query);
    let last_line_entity = current_lines_entities
        .iter()
        .max_by_key(|x| lines_query.get(**x).map(|y| y.row_index).unwrap_or(0));
    let last_text_entities = last_line_entity
        .map(|x| entity_get_children(x, &lines_children_query))
        .unwrap_or_default();

    let max_size = char_query
        .iter_many(last_text_entities)
        .map(|c| c.sections.first().map(|s| s.style.font_size).unwrap_or(0.))
        .collect::<Vec<f32>>()
        .iter()
        .fold(0., |acc: f32, e| acc.min(-*e));
    if let Some(mut line) = last_line_entity.and_then(|e| lines_query.get_mut(*e).ok()) {
        line.max_char_height = max_size;
    }
}

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

fn get_next_text(mut text: ResMut<LoadedText>) -> Option<char> {
    text.char_list.pop()
}
