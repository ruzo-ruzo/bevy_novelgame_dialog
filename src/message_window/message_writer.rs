use bevy::{prelude::*, sprite::Anchor, text::Font};

use super::*;
use crate::utility::*;
use page_scroller::*;
use window_controller::*;

#[derive(Component, Debug)]
pub struct MessageTextArea {
    pub area_size: Vec2,
}

#[derive(Component, Debug)]
pub struct MessageTextLine {
    pub row_index: usize,
    pub max_char_height: f32,
}

#[derive(Component, Debug)]
pub struct MessageTextChar {
    line_index: usize,
}

pub fn update_message(
    commands: Commands,
    sub_commands: Commands,
    fonts: Res<Assets<Font>>,
    mut config: ResMut<MessageWindowConfig>,
    window_children_query: Query<&Children, With<CurrentMessageWindow>>,
    mta_children_query: Query<&Children, With<MessageTextArea>>,
    lines_children_query: Query<&Children, With<MessageTextLine>>,
    mta_query: Query<&MessageTextArea>,
    lines_query: Query<(Entity, &MessageTextLine)>,
    lines_query_transform: Query<(Entity, &MessageTextLine, &Transform)>,
    chars_query: Query<(&MessageTextChar, &Transform, &Text)>,
    loded_text: ResMut<LoadedText>,
    time: Res<Time>,
    mut timer_config: ResMut<MessageDisplayConfig>,
    mut ps_event: EventWriter<PageScrollEvent>,
    mut pending: Local<Option<char>>,
) {
    timer_config.char_add_timer.tick(time.delta());
    if config.state == MessageWindowState::Writing && timer_config.char_add_timer.finished() {
        let mut new_text = *pending;
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
                let current_mta_entity = get_current_mta_entity(window_children_query);
                let last_line_entity = get_last_line_entity(
                    current_mta_entity,
                    lines_query.iter(),
                    mta_children_query,
                );
                let is_new_char_added = type_character(
                    commands,
                    sub_commands,
                    fonts,
                    config.reborrow(),
                    current_mta_entity,
                    last_line_entity,
                    lines_children_query,
                    mta_query,
                    lines_query_transform,
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

fn get_current_mta_entity(
    window_children_query: Query<&Children, With<CurrentMessageWindow>>,
) -> Option<Entity> {
    get_child_entities(window_children_query.iter())
        .first()
        .copied()
}

fn get_last_line_entity<'a, I: Iterator<Item = (Entity, &'a MessageTextLine)>>(
    current_mta_entity: Option<Entity>,
    lines_query: I,
    mta_children_query: Query<&Children, With<MessageTextArea>>,
) -> Option<Entity> {
    let current_lines_entities = current_mta_entity
        .map(|e| entity_get_children(&e, &mta_children_query))
        .unwrap_or_default();
    let current_lines = lines_query.filter(|l| current_lines_entities.iter().any(|e| l.0 == *e));
    current_lines.max_by_key(|x| x.1.row_index).map(|x| x.0)
}

fn type_character(
    commands: Commands,
    sub_commands: Commands,
    fonts: Res<Assets<Font>>,
    mut config: Mut<'_, MessageWindowConfig>,
    current_mta_entity: Option<Entity>,
    last_line_entity: Option<Entity>,
    lines_children_query: Query<&Children, With<MessageTextLine>>,
    mta_query: Query<&MessageTextArea>,
    lines_query: Query<(Entity, &MessageTextLine, &Transform)>,
    chars_query: Query<(&MessageTextChar, &Transform, &Text)>,
    new_char: char,
) -> bool {
    let last_text_entities = last_line_entity
        .map(|x| entity_get_children(&x, &lines_children_query))
        .unwrap_or_default();
    let last_text = chars_query.iter_many(last_text_entities.iter());
    let last_char = last_text.max_by_key(|x| x.0.line_index);
    let last_char_ts = last_char.and_then(|c| c.2.sections.first());
    let left_end = last_char.map(|c| c.1.translation.x).unwrap_or(0.)
        + last_char_ts.map(|s| s.style.font_size).unwrap_or(0.)
        + config.current_text_style.font_size;
    let area_width = current_mta_entity
        .and_then(|e| mta_query.get(e).ok())
        .map(|a| a.area_size.x);
    let is_linefeed = new_char == '\n';
    let is_overflow = left_end > area_width.unwrap_or(0.) || last_line_entity.is_none();

    print!("{}", new_char);
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

    let next_line = if is_overflow || is_linefeed {
        add_empty_line(
            sub_commands,
            config.reborrow(),
            current_mta_entity,
            last_line_entity,
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
        add_new_char(
            commands,
            fonts,
            config.reborrow(),
            new_char,
            last_char_x,
            next_line_index,
            next_line,
        );
    }
    true
}

fn add_new_char(
    mut commands: Commands,
    fonts: Res<Assets<Font>>,
    config: Mut<'_, MessageWindowConfig>,
    new_char: char,
    last_char_x: f32,
    next_line_index: usize,
    next_line: Option<Entity>,
) {
    let text_style = TextStyle {
        font: choice_font(&config.font_handles, new_char, fonts).unwrap_or_default(),
        ..config.current_text_style
    };
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

fn add_empty_line(
    mut commands: Commands,
    config: Mut<'_, MessageWindowConfig>,
    current_mta_entity: Option<Entity>,
    last_line_entity: Option<Entity>,
    mta_query: Query<&MessageTextArea>,
    lines_query: Query<(Entity, &MessageTextLine, &Transform)>,
) -> Option<Entity> {
    let last_line = last_line_entity.and_then(|x| lines_query.get(x).ok());
    let next_lines_origin_height = last_line
        .map(|x| x.2.translation.y + x.1.max_char_height)
        .unwrap_or(0.);
    let next_index = last_line.map(|x| x.1.row_index).unwrap_or(0) + 1;

    let area_height = current_mta_entity
        .and_then(|e| mta_query.get(e).ok())
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
    if let Some(mta) = current_mta_entity {
        commands.entity(mta).add_child(mtl);
    }
    Some(mtl)
}

pub fn update_line_height(
    window_children_query: Query<&Children, With<CurrentMessageWindow>>,
    mta_children_query: Query<&Children, With<MessageTextArea>>,
    lines_children_query: Query<&Children, With<MessageTextLine>>,
    char_query: Query<&Text, With<MessageTextChar>>,
    mut lines_query: Query<(Entity, &mut MessageTextLine)>,
) {
    let current_mta_entity = get_current_mta_entity(window_children_query);
    let last_line_entity =
        get_last_line_entity(current_mta_entity, lines_query.iter(), mta_children_query);
    let last_text_entities = last_line_entity
        .as_ref()
        .map(|x| entity_get_children(x, &lines_children_query))
        .unwrap_or_default();

    let max_size = char_query
        .iter_many(last_text_entities)
        .map(|c| c.sections.first().map(|s| s.style.font_size).unwrap_or(0.))
        .collect::<Vec<f32>>()
        .iter()
        .fold(0., |acc: f32, e| acc.min(-*e));
    if let Some(mut line) = last_line_entity.and_then(|e| lines_query.get_mut(e).ok().map(|x| x.1))
    {
        line.max_char_height = max_size;
    }
}

fn get_next_text(mut text: ResMut<LoadedText>) -> Option<char> {
    text.char_list.pop()
}
