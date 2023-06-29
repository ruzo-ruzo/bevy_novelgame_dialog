use super::super::*;
use super::feed_animation::*;

#[derive(Reflect, Default, Debug)]
pub struct InputForFeeding {
    pub target_text_box: Option<Entity>,
}

#[allow(clippy::type_complexity)]
pub fn skip_or_next(
    mut commands: Commands,
    mut waiting_text_query: Query<
        (Entity, &mut Visibility, &mut Transform, &Parent),
        With<MessageTextChar>,
    >,
    mut typing_texts_query: Query<(Entity, &mut TypingStyle, &Parent), With<MessageTextChar>>,
    text_box_query: Query<(&GlobalTransform, &Sprite)>,
    line_query: Query<(Entity, &Parent), With<MessageTextLine>>,
    mut icon_query: Query<(Entity, &mut Visibility), (With<WatingIcon>, Without<MessageTextChar>)>,
    mut bms_reader: EventReader<BMSEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in bms_reader.iter() {
        if let Some(InputForSkipping {
            next_event_ron: ron,
            target_text_box: Some(tb_entity),
        }) = event_wrapper.get_opt::<InputForSkipping>()
        {
            let mut typed_count = 0usize;
            let mut text_count = 0usize;
            for (text_entity, ts, t_parent) in &mut typing_texts_query {
                if line_query.get(t_parent.get()).map(|x| x.1.get()) == Ok(tb_entity) {
                    match *ts {
                        TypingStyle::Typed => {
                            typed_count += 1;
                        }
                        _ => {
                            commands.entity(text_entity).remove::<TypingStyle>();
                            commands.entity(text_entity).insert(TypingStyle::Typed);
                        }
                    }
                    text_count += 1;
                }
            }
            for (ic_entity, mut ic_vis) in &mut icon_query {
                *ic_vis = Visibility::Inherited;
                commands.entity(ic_entity).remove::<TypingTimer>();
            }
            if let Ok((tb_tf, tb_sp)) = text_box_query.get(tb_entity) {
                if text_count == typed_count {
                    if let Ok(ref_value) = read_ron(&type_registry, ron.clone()) {
                        commands.add(|w: &mut World| {
                            w.send_event(BMSEvent { value: ref_value });
                        })
                    }
                } else {
                    commands.entity(tb_entity).insert(make_wig_for_textbox(
                        tb_entity,
                        tb_tf,
                        tb_sp,
                        &type_registry,
                    ));
                }
            }
            for (text_entity, mut t_vis, mut tf, t_parent) in &mut waiting_text_query {
                if line_query.get(t_parent.get()).map(|x| x.1.get()) == Ok(tb_entity) {
                    tf.scale = Vec3::ONE;
                    *t_vis = Visibility::Inherited;
                    commands.entity(text_entity).remove::<TypingTimer>();
                    commands.entity(text_entity).insert(TypingStyle::Typed);
                }
            }
        }
    }
}

pub fn make_wig_for_textbox(
    tb_entity: Entity,
    tb_tf: &GlobalTransform,
    tb_sp: &Sprite,
    type_registry: &AppTypeRegistry,
) -> WaitInputGo {
    let base_size = tb_sp.custom_size.unwrap_or_default();
    let bottom_left = Vec2::new(tb_tf.translation().x, tb_tf.translation().y - base_size.y);
    let top_right = Vec2::new(bottom_left.x + base_size.x, tb_tf.translation().y);
    let ron_iff_opt = write_ron(
        type_registry,
        InputForFeeding {
            target_text_box: Some(tb_entity),
        },
    );
    let ron_ifs_opt = write_ron(
        type_registry,
        InputForSkipping {
            next_event_ron: ron_iff_opt.unwrap_or_default(),
            target_text_box: Some(tb_entity),
        },
    );
    WaitInputGo {
        ron: ron_ifs_opt.unwrap_or_default(),
        area: Rect::from_corners(bottom_left, top_right),
    }
}
