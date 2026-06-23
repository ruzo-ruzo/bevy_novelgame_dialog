use super::*;

// 流石に分割した方がいい気もする
pub(super) fn shift_selected(
    mut commands: Commands,
    selective_query: Query<
        (Entity, &Selective, &TextArea, &WaitInputGo, &ChildOf),
        Without<Pending>,
    >,
    selected_query: Query<Entity, (With<Selected>, Without<Pending>)>,
    writing_query: Query<&DialogBox>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DialogBoxCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut select_event: MessageWriter<ButtonIsSelected>,
) {
    let mut next_select_opt: Option<Entity> = None;
    let pointed_opt = camera_query
        .single()
        .ok()
        .zip(window_query.single().ok().and_then(|y| y.cursor_position()))
        .and_then(|(c, p)| c.0.viewport_to_world_2d(c.1, p).ok());
    for (target_entity, _, _, wig, _) in &selective_query {
        if pointed_opt.is_some_and(|x| wig.area.contains(x)) {
            next_select_opt = Some(target_entity);
        }
    }
    let selected_res = selected_query.single();
    let mut vertical_targets = selective_query
        .iter()
        .filter(|x| x.1.key_vector == SelectVector::Vertical)
        .collect::<Vec<_>>();
    let mut horizen_targets = selective_query
        .iter()
        .filter(|x| x.1.key_vector == SelectVector::Horizon)
        .collect::<Vec<_>>();
    vertical_targets.sort_by_key(|x| x.1.number);
    horizen_targets.sort_by_key(|x| x.1.number);
    let gamepad = gamepads.iter().next();
    if (keys.any_just_pressed([KeyCode::ArrowUp]))
        || gamepad.is_some_and(|x| x.just_pressed(GamepadButton::DPadUp))
    {
        if let Ok(selected_entity) = selected_res {
            if let Some((
                _,
                Selective {
                    number: selected_num,
                    key_vector: SelectVector::Vertical,
                },
                ..,
            )) = vertical_targets.iter().find(|x| x.0 == selected_entity)
            {
                let max_size = vertical_targets.iter().len() - 1;
                let next_num = if *selected_num == 0 {
                    max_size
                } else {
                    selected_num - 1
                };
                let next_opt = vertical_targets.iter().find(|x| x.1.number == next_num);
                next_select_opt = next_opt.map(|x| x.0);
            }
        } else {
            next_select_opt = vertical_targets
                .iter()
                .map(|x| x.0)
                .collect::<Vec<_>>()
                .last()
                .copied();
        }
    }
    if (keys.any_just_pressed([KeyCode::ArrowDown]))
        || gamepad.is_some_and(|x| x.just_pressed(GamepadButton::DPadDown))
    {
        if let Ok(selected_entity) = selected_res {
            if let Some((
                _,
                Selective {
                    number: selected_num,
                    key_vector: SelectVector::Vertical,
                },
                ..,
            )) = vertical_targets.iter().find(|x| x.0 == selected_entity)
            {
                let max_size = vertical_targets.iter().len() - 1;
                let next_num = if *selected_num >= max_size {
                    0
                } else {
                    selected_num + 1
                };
                let next_opt = vertical_targets.iter().find(|x| x.1.number == next_num);
                next_select_opt = next_opt.map(|x| x.0);
            }
        } else {
            next_select_opt = vertical_targets
                .iter()
                .map(|x| x.0)
                .collect::<Vec<_>>()
                .first()
                .copied();
        }
    }
    if (keys.any_just_pressed([KeyCode::ArrowLeft]))
        || gamepad.is_some_and(|x| x.just_pressed(GamepadButton::DPadLeft))
    {
        if let Ok(selected_entity) = selected_res {
            if let Some((
                _,
                Selective {
                    number: selected_num,
                    key_vector: SelectVector::Horizon,
                },
                ..,
            )) = horizen_targets.iter().find(|x| x.0 == selected_entity)
            {
                let max_size = horizen_targets.iter().len() - 1;
                let next_num = if *selected_num == 0 {
                    max_size
                } else {
                    selected_num - 1
                };
                let next_opt = horizen_targets.iter().find(|x| x.1.number == next_num);
                next_select_opt = next_opt.map(|x| x.0);
            }
        } else {
            next_select_opt = horizen_targets
                .iter()
                .map(|x| x.0)
                .collect::<Vec<_>>()
                .last()
                .copied();
        }
    }
    if (keys.any_just_pressed([KeyCode::ArrowRight]))
        || gamepad.is_some_and(|x| x.just_pressed(GamepadButton::DPadRight))
    {
        if let Ok(selected_entity) = selected_res {
            if let Some((
                _,
                Selective {
                    number: selected_num,
                    key_vector: SelectVector::Horizon,
                },
                ..,
            )) = horizen_targets.iter().find(|x| x.0 == selected_entity)
            {
                let max_size = horizen_targets.iter().len() - 1;
                let next_num = if *selected_num >= max_size {
                    0
                } else {
                    selected_num + 1
                };
                let next_opt = horizen_targets.iter().find(|x| x.1.number == next_num);
                next_select_opt = next_opt.map(|x| x.0);
            }
        } else {
            next_select_opt = horizen_targets
                .iter()
                .map(|x| x.0)
                .collect::<Vec<_>>()
                .first()
                .copied();
        }
    }
    if let Some(next_entity) = next_select_opt {
        if let Ok(selected_entity) = selected_res {
            commands.entity(selected_entity).remove::<Selected>();
        }
        commands.entity(next_entity).insert(Selected);
        if let Ok((_, selective, ta, _, parent)) = selective_query.get(next_entity) {
            let db_name_opt = writing_query.get(parent.parent()).map(|x| x.name.clone());
            let event = ButtonIsSelected {
                writing_name: db_name_opt.unwrap_or_default(),
                text_area_name: ta.name.clone(),
                select_vector: selective.key_vector,
                select_number: selective.number,
            };
            select_event.write(event);
        }
    }
}
