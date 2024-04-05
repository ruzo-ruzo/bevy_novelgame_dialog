use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::view::{RenderLayers, Visibility},
    sprite::Anchor,
    text::JustifyText,
};

pub mod feed_animation;
pub mod typing_animations;

use super::*;
use crate::utility::*;
use feed_animation::*;

#[derive(Component, Debug)]
pub struct MessageTextLine {
    alignment: JustifyText,
}

#[derive(Component, Debug)]
pub struct MessageTextChar;

#[derive(Bundle, Debug)]
struct CharBundle {
    text_char: MessageTextChar,
    timer: TypingTimer,
    text2d: Text2dBundle,
    layer: RenderLayers,
    writing: WritingStyle,
}

#[derive(Bundle)]
struct LineBundle {
    line: MessageTextLine,
    sprites: SpriteBundle,
}

#[derive(Component, Clone, Debug)]
pub struct TypingTimer {
    pub timer: Timer,
}

#[derive(SystemParam, Debug)]
#[allow(clippy::type_complexity)]
pub struct LastTextData<'w, 's> {
    text: Query<'w, 's, LastText, (With<Current>, With<MessageTextChar>)>,
    line: Query<
        'w,
        's,
        (Entity, &'static Transform, &'static Sprite, &'static Parent),
        (With<Current>, With<MessageTextLine>),
    >,
}

type LastText = (
    Entity,
    &'static Transform,
    &'static Text,
    &'static TypingTimer,
    &'static Parent,
);

type TextAreaData<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Sprite,
        &'static TypeTextConfig,
        &'static Parent,
    ),
    (With<Current>, With<TextArea>),
>;

pub fn add_new_text(
    mut commands: Commands,
    mut dialog_box_query: Query<(Entity, &mut LoadedScript, &mut DialogBoxPhase)>,
    text_area_query: TextAreaData,
    last_data: LastTextData,
    app_type_registry: Res<AppTypeRegistry>,
    mut wrapper: EventWriter<BdsEvent>,
    mut ps_event: EventWriter<FeedWaitingEvent>,
    fonts: Res<Assets<Font>>,
    mut pending: Local<Option<Order>>,
    mut in_cr: Local<bool>,
) {
    for (w_ent, mut script, mut ws) in &mut dialog_box_query {
        if *ws != DialogBoxPhase::Typing {
            continue;
        }
        for (tb_ent, tb_spr, config, parent) in &text_area_query {
            if w_ent != parent.get() {
                continue;
            }
            let (mut last_line_opt, mut last_text_opt, mut last_x, mut last_y, mut last_timer) =
                initialize_typing_data(&last_data, tb_ent);
            let Vec2 {
                x: max_width,
                y: max_height,
            } = tb_spr.custom_size.unwrap_or_default();
            loop {
                let next_order = get_next_order(&pending, &mut script.order_list, *in_cr);
                match next_order {
                    Some(Order::Type {
                        character: new_word,
                    }) => {
                        let new_text_opt = make_new_text(
                            new_word,
                            config,
                            &mut last_x,
                            last_y,
                            &mut last_timer,
                            fonts.as_ref(),
                            max_width,
                        );
                        let (Some(new_text), Some(last_line)) = (new_text_opt, last_line_opt)
                        else {
                            *pending = next_order;
                            *in_cr = true;
                            continue;
                        };
                        let new_text_entity = commands.spawn((new_text, Current)).id();
                        if let Some(last_text) = last_text_opt {
                            commands.entity(last_text).remove::<Current>();
                        }
                        last_text_opt = Some(new_text_entity);
                        commands.entity(last_line).add_child(new_text_entity);
                        *pending = None;
                        *in_cr = false;
                    }
                    Some(Order::CarriageReturn) => {
                        let new_line_opt =
                            make_empty_line(config, &mut last_x, &mut last_y, max_height);
                        let Some(new_line) = new_line_opt else {
                            send_feed_event(
                                &mut ps_event,
                                w_ent,
                                &last_timer,
                                &mut ws,
                                last_x,
                                last_y,
                            );
                            *in_cr = true;
                            break;
                        };
                        let new_line_entity = commands.spawn((new_line, Current)).id();
                        if let Some(last_line) = last_line_opt {
                            commands.entity(last_line).remove::<Current>();
                        }
                        last_line_opt = Some(new_line_entity);
                        commands.entity(tb_ent).add_child(new_line_entity);
                        *in_cr = false;
                        continue;
                    }
                    Some(Order::PageFeed) => {
                        send_feed_event(&mut ps_event, w_ent, &last_timer, &mut ws, last_x, last_y);
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

pub fn initialize_typing_data(
    last_data: &LastTextData,
    text_box_entity: Entity,
) -> (Option<Entity>, Option<Entity>, f32, f32, TypingTimer) {
    let last_line_data_opt = last_data.line.iter().find(|x| x.3.get() == text_box_entity);
    let last_line_opt = last_line_data_opt.map(|x| x.0);
    let last_text_data_opt = last_data
        .text
        .iter()
        .filter(|x| Some(x.4.get()) == last_line_opt)
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
                .map(|x| x.3.timer.remaining_secs())
                .unwrap_or_default(),
            TimerMode::Once,
        ),
    };
    let last_x = last_text_data_opt
        .map(|t| t.1.translation.x)
        .unwrap_or_default();
    let last_y = last_line_data_opt
        .map(|l| l.1.translation.y)
        .unwrap_or_default();
    (last_line_opt, last_text_opt, last_x, last_y, last_timer)
}

fn send_feed_event(
    fw_event: &mut EventWriter<FeedWaitingEvent>,
    entity: Entity,
    last_timer: &TypingTimer,
    ws: &mut DialogBoxPhase,
    last_x: f32,
    last_y: f32,
) {
    fw_event.send(FeedWaitingEvent {
        target_window: entity,
        wait_sec: last_timer.timer.remaining_secs(),
        last_pos: Vec2::new(last_x, last_y),
    });
    *ws = DialogBoxPhase::WaitingAction;
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

//Todo: カーニングつける。
fn make_new_text(
    new_word: char,
    config: &TypeTextConfig,
    last_x: &mut f32,
    last_y: f32,
    last_timer: &mut TypingTimer,
    font_assets: &Assets<Font>,
    max_width: f32,
) -> Option<CharBundle> {
    let next_x = *last_x + config.text_style.font_size;
    if next_x > max_width {
        None
    } else {
        let text_style = TextStyle {
            font: choice_font(&config.fonts, new_word, font_assets).unwrap_or_default(),
            ..config.text_style
        };
        let text2d_bundle = Text2dBundle {
            text: Text::from_section(new_word.to_string(), text_style),
            transform: Transform::from_translation(Vec3::new(next_x, 0., 0.)),
            visibility: Visibility::Hidden,
            text_anchor: Anchor::BottomLeft,
            ..default()
        };
        let last_secs = last_timer.timer.remaining_secs();
        let type_sec = match config.typing_timing {
            TypingTiming::ByChar { sec: s } => last_secs + s,
            TypingTiming::ByLine { sec: s } => {
                let is_first_char = last_y >= -config.text_style.font_size;
                last_secs
                    + if *last_x == 0. && !is_first_char {
                        s
                    } else {
                        0.
                    }
            }
            _ => 0.,
        };
        let typing_timer = TypingTimer {
            timer: Timer::from_seconds(type_sec, TimerMode::Once),
        };
        *last_x += config.text_style.font_size;
        *last_timer = typing_timer.clone();
        Some(CharBundle {
            text_char: MessageTextChar,
            timer: typing_timer,
            text2d: text2d_bundle,
            layer: config.layer,
            writing: config.writing,
        })
    }
}

//Todo: 高さ調整つける
fn make_empty_line(
    config: &TypeTextConfig,
    last_x: &mut f32,
    last_y: &mut f32,
    min_height: f32,
) -> Option<LineBundle> {
    *last_x = 0.;
    *last_y -= config.text_style.font_size;
    if *last_y < -min_height {
        None
    } else {
        let sprite_bundle = SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., *last_y, config.pos_z)),
            ..default()
        };
        Some(LineBundle {
            sprites: sprite_bundle,
            line: MessageTextLine {
                alignment: config.alignment,
            },
        })
    }
}

// Todo: 高さのセンタリングも出来るようにする
pub fn settle_lines(
    dialogbox_query: Query<(Entity, &DialogBoxPhase), With<DialogBox>>,
    mut text_lines: Query<(&MessageTextLine, &mut Transform)>,
    text_char: Query<&Text, With<MessageTextChar>>,
    mut sprite_query: Query<&mut Sprite>,
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
            for tl_entity in tl_entities {
                let Ok((mtl, mut l_tf)) = text_lines.get_mut(*tl_entity) else {
                    continue;
                };
                let Ok(mut tl_spr) = sprite_query.get_mut(*tl_entity) else {
                    continue;
                };
                let Ok(tx_entities) = children_query.get(*tl_entity) else {
                    continue;
                };
                let mut text_size_list: Vec<f32> = Vec::new();
                for tx_entity in tx_entities {
                    let Ok(text) = text_char.get(*tx_entity) else {
                        continue;
                    };
                    let text_size = text.sections.first().map(|x| x.style.font_size);
                    text_size_list.push(text_size.unwrap_or_default());
                }
                let base_hight = tl_spr.custom_size.map(|x| x.y).unwrap_or_default();
                let line_width: f32 = text_size_list.iter().sum();
                let line_height = text_size_list
                    .iter()
                    .reduce(|x, y| if x > y { x } else { y })
                    .unwrap_or(&base_hight);
                prev_height -= line_height;
                tl_spr.custom_size = Some(Vec2::new(line_width, *line_height));
                if *phase != DialogBoxPhase::Typing {
                    continue;
                }
                let Ok(ta_spr) = sprite_query.get(*ta_entity) else {
                    continue;
                };
                let area_width = ta_spr.custom_size.map(|s| s.x).unwrap_or_default();
                l_tf.translation.x = match mtl.alignment {
                    JustifyText::Center => (area_width - line_width) / 2.,
                    JustifyText::Right => area_width - line_width,
                    _ => 0.,
                };
                l_tf.translation.y = prev_height
            }
        }
    }
}
