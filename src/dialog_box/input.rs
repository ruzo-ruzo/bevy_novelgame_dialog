use crate::dialog_box::public::configs::*;
use crate::dialog_box::public::events::*;
use crate::dialog_box::window_controller::*;
use crate::dialog_box::DialogBoxCamera;
use crate::read_script::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// ゲームパッド持ってないので全体的に挙動が未確認

#[derive(Component, Debug)]
pub struct Selected;

#[derive(Component, Debug)]
pub struct WaitInputGo {
    pub ron: String,
    pub area: Rect,
}

#[derive(Component)]
pub struct Selective {
    pub key_vector: SelectVector,
    pub number: usize,
}

// ToDo: 長押しで連続スキップできるようにしときたい
#[allow(clippy::nonminimal_bool)]
pub fn go_selected(
    mut commands: Commands,
    target_query: Query<(Entity, &WaitInputGo, &TextArea, &Parent), Without<Pending>>,
    dialog_box_query: Query<&DialogBox>,
    selected_query: Query<Entity, (With<Selected>, Without<Pending>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DialogBoxCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    mut bds_event: EventWriter<BdsEvent>,
    mut go_event: EventWriter<GoSelectedEvent>,
    gamepads: Res<Gamepads>,
    type_registry: Res<AppTypeRegistry>,
) {
    let pointed_opt = camera_query
        .get_single()
        .ok()
        .and_then(|x| {
            window_query
                .get_single()
                .ok()
                .and_then(|y| y.cursor_position())
                .map(|y| (x, y))
        })
        .and_then(|(c, p)| c.0.viewport_to_world_2d(c.1, p));
    for (target_entity, wig, ta, ta_parent) in &target_query {
        let mut touched_position_list = touches
            .iter_just_pressed()
            .filter_map(|t| camera_query.get_single().ok().map(|c| (c, t)))
            .filter_map(|(c, t)| c.0.viewport_to_world_2d(c.1, t.position()));
        let is_selected = selected_query
            .get_single()
            .is_ok_and(|e| e == target_entity);
        let is_pointed = pointed_opt.is_some_and(|x| (wig.area.contains(x)));
        let gamepad_go_button = gamepads.iter().next().map(|x| GamepadButton {
            gamepad: x,
            button_type: GamepadButtonType::South,
        });
        if (keys.any_just_pressed([KeyCode::Space, KeyCode::Enter, KeyCode::NumpadEnter])
            && is_selected)
            || (gamepad_go_button.is_some_and(|x| gamepad_buttons.just_pressed(x)) && is_selected)
            || (mouse_buttons.just_pressed(MouseButton::Left) && is_pointed)
            || touched_position_list.any(|t| wig.area.contains(t))
        {
            if let Ok(ref_value) = read_ron(&type_registry, wig.ron.clone()) {
                bds_event.send(BdsEvent { value: ref_value });
            }
            let db_name_opt = dialog_box_query
                .get(ta_parent.get())
                .map(|x| x.name.clone());
            go_event.send(GoSelectedEvent {
                dialog_box_name: db_name_opt.unwrap_or_default(),
                text_area_name: ta.name.clone(),
            });
            commands.entity(target_entity).remove::<WaitInputGo>();
        }
    }
}

// 流石に分割した方がいい気もする
pub fn shift_selected(
    mut commands: Commands,
    selective_query: Query<
        (Entity, &Selective, &TextArea, &WaitInputGo, &Parent),
        Without<Pending>,
    >,
    selected_query: Query<Entity, (With<Selected>, Without<Pending>)>,
    dialog_box_query: Query<&DialogBox>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DialogBoxCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    gamepads: Res<Gamepads>,
    mut select_event: EventWriter<SelectedEvent>,
) {
    let mut next_select_opt: Option<Entity> = None;
    let pointed_opt = camera_query
        .get_single()
        .ok()
        .and_then(|x| {
            window_query
                .get_single()
                .ok()
                .and_then(|y| y.cursor_position())
                .map(|y| (x, y))
        })
        .and_then(|(c, p)| c.0.viewport_to_world_2d(c.1, p));
    for (target_entity, _, _, wig, _) in &selective_query {
        if pointed_opt.is_some_and(|x| (wig.area.contains(x))) {
            next_select_opt = Some(target_entity);
        }
    }
    let selected_res = selected_query.get_single();
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
    let gamepad_up_button = gamepads.iter().next().map(|x| GamepadButton {
        gamepad: x,
        button_type: GamepadButtonType::DPadUp,
    });
    let gamepad_down_button = gamepads.iter().next().map(|x| GamepadButton {
        gamepad: x,
        button_type: GamepadButtonType::DPadDown,
    });
    let gamepad_left_button = gamepads.iter().next().map(|x| GamepadButton {
        gamepad: x,
        button_type: GamepadButtonType::DPadLeft,
    });
    let gamepad_right_button = gamepads.iter().next().map(|x| GamepadButton {
        gamepad: x,
        button_type: GamepadButtonType::DPadRight,
    });
    if (keys.any_just_pressed([KeyCode::ArrowUp]))
        || (gamepad_up_button.is_some_and(|x| gamepad_buttons.just_pressed(x)))
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
        || (gamepad_down_button.is_some_and(|x| gamepad_buttons.just_pressed(x)))
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
        || (gamepad_left_button.is_some_and(|x| gamepad_buttons.just_pressed(x)))
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
        || (gamepad_right_button.is_some_and(|x| gamepad_buttons.just_pressed(x)))
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
            let db_name_opt = dialog_box_query.get(parent.get()).map(|x| x.name.clone());
            let event = SelectedEvent {
                dialog_box_name: db_name_opt.unwrap_or_default(),
                text_area_name: ta.name.clone(),
                select_vector: selective.key_vector,
                select_number: selective.number,
            };
            select_event.send(event);
        }
    }
}
