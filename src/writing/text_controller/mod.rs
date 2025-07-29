use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::view::{RenderLayers, Visibility},
    sprite::Anchor,
};
use rustybuzz::Face;

pub(super) mod feed_animation;
pub(super) mod typing_animations;

use super::*;
use crate::utility::*;
use feed_animation::*;

#[derive(Component)]
#[require(Sprite)]
pub(in crate::writing) struct MessageTextLine {
    horizon_alignment: AlignHorizon,
    vertical_alignment: AlignVertical,
}

#[derive(Component, Debug)]
#[require(TypingTimer, Text2d, RenderLayers, WritingStyle)]
pub(in crate::writing) struct MessageTextChar;

#[derive(Component, Default, Clone, Debug)]
pub(super) struct TypingTimer {
    pub timer: Timer,
}

pub(super) struct CharPos {
    pub x: f32,
    pub y: f32,
}

pub(super) struct LastChar {
    pub entity: Option<Entity>,
    pub pos: CharPos,
    pub timer: TypingTimer,
}

#[derive(SystemParam, Debug)]
#[allow(clippy::type_complexity)]
pub(super) struct TextQuery<'w, 's> {
    text: Query<'w, 's, CharData, (With<Current>, With<MessageTextChar>)>,
    line: Query<'w, 's, LineData, (With<Current>, With<MessageTextLine>)>,
}

type CharData = (
    Entity,
    &'static Transform,
    &'static Text,
    &'static TextFont,
    &'static TypingTimer,
    &'static Parent,
);
type LineData = (Entity, &'static Transform, &'static Sprite, &'static Parent);

#[derive(SystemParam, Debug)]
pub(super) struct CurrentTextAreaQuery<'w, 's> {
    area: Query<'w, 's, AreaData, (With<Current>, With<TextArea>)>,
}

type AreaData = (
    Entity,
    &'static Sprite,
    &'static TypeTextConfig,
    &'static Parent,
);

pub(in crate::writing) fn add_new_text(
    mut commands: Commands,
    mut writing_query: Query<(Entity, &DialogBox, &mut LoadedScript, &mut DialogBoxPhase)>,
    text_area_query: CurrentTextAreaQuery,
    last_data: TextQuery,
    app_type_registry: Res<AppTypeRegistry>,
    mut wrapper: EventWriter<BdsEvent>,
    mut ps_event: EventWriter<FeedWaitingEvent>,
    fonts_res: Res<Assets<Font>>,
    mut pending: Local<Option<Order>>,
    mut in_cr: Local<bool>,
) {
    for (w_ent, DialogBox { name: w_name }, mut script, mut dbp) in &mut writing_query {
        if *dbp != DialogBoxPhase::Typing {
            continue;
        }
        for (tb_ent, tb_spr, config, parent) in &text_area_query.area {
            if w_ent != parent.get() {
                continue;
            }
            let (mut last_line_opt, mut last_char) = initialize_typing_data(&last_data, tb_ent);
            let Vec2 {
                x: width,
                y: height,
            } = tb_spr.custom_size.unwrap_or_default();
            loop {
                let next_order = get_next_order(&pending, &mut script.order_list, *in_cr);
                match next_order {
                    Some(Order::Type {
                        character: new_word,
                    }) => {
                        let fonts = fonts_res.as_ref();
                        let char_config = (config, &mut last_char, fonts, width, last_line_opt);
                        if add_char(&mut commands, new_word, char_config) {
                            *pending = None;
                            *in_cr = false;
                        } else {
                            *pending = next_order;
                            *in_cr = true;
                        };
                    }
                    Some(Order::CarriageReturn) => {
                        let line_config = (config, &mut last_char, height, &mut last_line_opt);
                        if add_empty_line(&mut commands, line_config, tb_ent) {
                            *in_cr = false;
                        } else {
                            send_feed_event(&mut ps_event, w_name, &last_char, &mut dbp);
                            *in_cr = true;
                            break;
                        };
                    }
                    Some(Order::PageFeed) => {
                        send_feed_event(&mut ps_event, w_name, &last_char, &mut dbp);
                        *in_cr = true;
                        break;
                    }
                    Some(Order::ThroghEvent { ron: r }) => {
                        let event_opt = read_ron(&app_type_registry, r);
                        if let Ok(reflect_value) = event_opt {
                            wrapper.send(BdsEvent {
                                value: reflect_value,
                            });
                        }
                        break;
                    }
                    None => break,
                }
            }
        }
    }
}

pub(in crate::writing) fn initialize_typing_data(
    last_data: &TextQuery,
    text_box_entity: Entity,
) -> (Option<Entity>, LastChar) {
    let last_line_data_opt = last_data.line.iter().find(|x| x.3.get() == text_box_entity);
    let last_line_opt = last_line_data_opt.map(|x| x.0);
    let last_text_data_opt = last_data
        .text
        .iter()
        .filter(|x| Some(x.5.get()) == last_line_opt)
        .max_by(|x, y| {
            if x.1.translation.x >= y.1.translation.x {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        });
    let last_text_opt = last_text_data_opt.map(|x| x.0);
    let last_timer = TypingTimer {
        timer: Timer::from_seconds(
            last_text_data_opt
                .map(|x| x.4.timer.remaining_secs())
                .unwrap_or_default(),
            TimerMode::Once,
        ),
    };
    let last_x = last_text_data_opt
        .map(|t| t.3.font_size + t.1.translation.x)
        .unwrap_or_default();
    let last_y = last_line_data_opt
        .map(|l| l.1.translation.y)
        .unwrap_or_default();
    let char_pos = CharPos {
        x: last_x,
        y: last_y,
    };
    (
        last_line_opt,
        LastChar {
            entity: last_text_opt,
            pos: char_pos,
            timer: last_timer,
        },
    )
}

fn send_feed_event(
    fw_event: &mut EventWriter<FeedWaitingEvent>,
    name: &str,
    last_char: &LastChar,
    dbp: &mut DialogBoxPhase,
) {
    fw_event.send(FeedWaitingEvent {
        target_box_name: name.to_string(),
        wait_sec: last_char.timer.timer.remaining_secs(),
        // last_pos: Vec2::new(last_char.pos.x, last_char.pos.y),
    });
    *dbp = DialogBoxPhase::WaitingAction;
}

fn get_next_order(
    pending: &Option<Order>,
    order_list: &mut Option<Vec<Order>>,
    in_cr: bool,
) -> Option<Order> {
    match (pending, order_list, in_cr) {
        (_, _, true) => Some(Order::CarriageReturn),
        (s @ Some(_), _, _) => s.clone(),
        (None, Some(ref mut list), _) => list.pop(),
        _ => None,
    }
}

fn add_char(
    commands: &mut Commands,
    new_word: char,
    (config, last_char, font_assets, width, last_line_opt): (
        &TypeTextConfig,
        &mut LastChar,
        &Assets<Font>,
        f32,
        Option<Entity>,
    ),
) -> bool {
    let new_str = String::from(new_word);
    let font_h = choice_font(&config.fonts, new_word, font_assets);
    let size_coefficient = find_by_regex(new_str.clone(), &config.size_by_regulars).unwrap_or(1.0);
    let kerning_coefficient = find_by_regex(new_str, &config.kerning_by_regulars).unwrap_or(0.0);
    let true_size = config.text_font.font_size * size_coefficient;
    let kerning = true_size * kerning_coefficient;
    let target_x = last_char.pos.x + true_size + kerning;
    if target_x > width {
        false
    } else {
        let text_font = TextFont {
            font: font_h.clone().unwrap_or_default(),
            font_size: true_size,
            ..Default::default()
        };
        let text2d_bundle = (
            Text2d::new(new_word.to_string()),
            Transform::from_translation(Vec3::new(last_char.pos.x, 0.0, 0.0)),
            Visibility::Hidden,
            Anchor::BottomLeft,
            text_font,
            config.text_color,
        );
        let last_secs = last_char.timer.timer.remaining_secs();
        let type_sec = match config.typing_timing {
            TypingTiming::ByChar { sec: s } => last_secs + s,
            TypingTiming::ByLine { sec: s } => {
                let is_first_char = last_char.pos.y >= -true_size;
                if last_char.pos.x == 0. && !is_first_char {
                    last_secs + s
                } else {
                    last_secs + 0.0
                }
            }
            _ => 0.0,
        };
        let typing_timer = TypingTimer {
            timer: Timer::from_seconds(type_sec, TimerMode::Once),
        };
        let Some(font) = &font_assets.get(&font_h.unwrap_or_default()) else {
            return false;
        };
        let Some(glyph_buffer) = get_glyph_buffer(font, new_word) else {
            return false;
        };
        let Some(positions) = &glyph_buffer.glyph_positions().iter().next() else {
            return false;
        };
        let Some(face) = Face::from_slice(&font.data, 0) else {
            return false;
        };
        let pt_per_height = true_size / face.height() as f32;
        let advance = pt_per_height * positions.x_advance as f32;
        let next_x = last_char.pos.x + advance + kerning;
        last_char.pos.x = if config.monospace { target_x } else { next_x };
        last_char.timer = typing_timer.clone();
        let new_char = (
            MessageTextChar,
            typing_timer,
            text2d_bundle,
            config.layer.clone(),
            config.writing,
        );
        if let Some(last_line) = last_line_opt {
            let new_char_entity = commands.spawn((new_char, Current)).id();
            if let Some(last_text) = last_char.entity {
                commands.entity(last_text).remove::<Current>();
            }
            last_char.entity = Some(new_char_entity);
            commands.entity(last_line).add_child(new_char_entity);
            true
        } else {
            false
        }
    }
}

fn add_empty_line(
    commands: &mut Commands,
    (config, last_char, min_height, last_line_opt): (
        &TypeTextConfig,
        &mut LastChar,
        f32,
        &mut Option<Entity>,
    ),
    tb_ent: Entity,
) -> bool {
    last_char.pos.x = 0.;
    last_char.pos.y -= config.text_font.font_size;
    if last_char.pos.y < -min_height {
        false
    } else {
        let sprite_bundle = (
            Sprite {
                anchor: Anchor::BottomLeft,
                ..default()
            },
            Transform::from_translation(Vec3::new(0., last_char.pos.y, config.pos_z)),
        );
        let new_line = (
            sprite_bundle,
            MessageTextLine {
                horizon_alignment: config.horizon_alignment,
                vertical_alignment: config.vertical_alignment,
            },
        );
        let new_line_entity = commands.spawn((new_line, Current)).id();
        if let Some(last_line) = last_line_opt {
            commands.entity(*last_line).remove::<Current>();
        }
        *last_line_opt = Some(new_line_entity);
        commands.entity(tb_ent).add_child(new_line_entity);
        true
    }
}

pub(in crate::writing) fn settle_lines(
    dialogbox_query: Query<(Entity, &DialogBoxPhase), With<DialogBox>>,
    mut text_lines: Query<(&MessageTextLine, &mut Transform), Without<MessageTextChar>>,
    text_char: Query<(&TextFont, &Transform), With<MessageTextChar>>,
    area_sprite_query: Query<&mut Sprite, With<TextArea>>,
    mut line_sprite_query: Query<&mut Sprite, Without<TextArea>>,
    children_query: Query<&Children>,
) {
    for (db_entity, phase) in &dialogbox_query {
        let Ok(ta_entities) = children_query.get(db_entity) else {
            continue;
        };
        for ta_entity in ta_entities {
            let mut prev_height = 0f32;
            let Ok(tl_entities) = children_query.get(*ta_entity) else {
                continue;
            };
            let Ok(ta_spr) = area_sprite_query.get(*ta_entity) else {
                continue;
            };
            for tl_entity in tl_entities {
                let Ok((mtl, mut l_tf)) = text_lines.get_mut(*tl_entity) else {
                    continue;
                };
                let Ok(mut tl_spr) = line_sprite_query.get_mut(*tl_entity) else {
                    continue;
                };
                let Ok(tx_entities) = children_query.get(*tl_entity) else {
                    continue;
                };
                let mut text_size_list: Vec<f32> = Vec::new();
                let mut last_pos_x = 0.0;
                for tx_entity in tx_entities {
                    let Ok((text_font, t_tf)) = text_char.get(*tx_entity) else {
                        continue;
                    };
                    let text_size = text_font.font_size;
                    text_size_list.push(text_size);
                    if t_tf.translation.x >= last_pos_x {
                        last_pos_x = t_tf.translation.x + text_size;
                    }
                }
                let base_hight = tl_spr.custom_size.map(|x| x.y).unwrap_or_default();
                let line_width = last_pos_x;
                let line_height = text_size_list
                    .iter()
                    .reduce(|x, y| if x > y { x } else { y })
                    .unwrap_or(&base_hight);
                tl_spr.custom_size = Some(Vec2::new(line_width, *line_height));
                if *phase != DialogBoxPhase::Typing {
                    continue;
                }
                let area_width = ta_spr.custom_size.map(|s| s.x).unwrap_or_default();
                l_tf.translation.x = match mtl.horizon_alignment {
                    AlignHorizon::Center => (area_width - line_width) / 2.0,
                    AlignHorizon::Right => area_width - line_width,
                    _ => 0.0,
                };
                l_tf.translation.y = prev_height - line_height;
                prev_height -= line_height;
            }
            let area_height = ta_spr.custom_size.map(|s| s.y).unwrap_or_default();
            for tl_entity in tl_entities {
                if *phase != DialogBoxPhase::Typing {
                    continue;
                }
                if let Ok((mtl, mut l_tf)) = text_lines.get_mut(*tl_entity) {
                    l_tf.translation.y -= match mtl.vertical_alignment {
                        AlignVertical::Center => (area_height + prev_height) / 2.0,
                        AlignVertical::Bottom => area_height + prev_height,
                        _ => 0.0,
                    }
                }
            }
        }
    }
}
