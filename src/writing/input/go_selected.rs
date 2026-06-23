use super::*;

// ToDo: 長押しで連続スキップできるようにしときたい
#[allow(clippy::nonminimal_bool)]
pub(super) fn go_selected(
    mut commands: Commands,
    target_query: Query<(Entity, &WaitInputGo, &TextArea, &ChildOf), Without<Pending>>,
    writing_query: Query<&DialogBox>,
    selected_query: Query<Entity, (With<Selected>, Without<Pending>)>,
    selective_query: Query<Entity, (With<Selective>, Without<Pending>)>,
    pending_query: Query<(Entity, &Pending)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DialogBoxCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut bds_event: MessageWriter<BdsEvent>,
    mut go_event: MessageWriter<ButtonIsPushed>,
    gamepads: Query<&Gamepad>,
    type_registry: Res<AppTypeRegistry>,
) {
    let pointed_opt = camera_query
        .single()
        .ok()
        .zip(window_query.single().ok().and_then(|y| y.cursor_position()))
        .and_then(|(c, p)| c.0.viewport_to_world_2d(c.1, p).ok());
    for (target_entity, wig, ta, ta_parent) in &target_query {
        let mut touched_position_list = touches
            .iter_just_pressed()
            .filter_map(|t| camera_query.single().ok().map(|c| (c, t)))
            .filter_map(|(c, t)| c.0.viewport_to_world_2d(c.1, t.position()).ok());
        let is_selected = selected_query.single().is_ok_and(|e| e == target_entity);
        let is_pointed = pointed_opt.is_some_and(|x| wig.area.contains(x));
        let gamepad = gamepads.iter().next();
        if (keys.any_just_pressed([KeyCode::Space, KeyCode::Enter, KeyCode::NumpadEnter])
            && is_selected)
            || (gamepad
                .map(|x| x.just_pressed(GamepadButton::South))
                .is_some()
                && is_selected)
            || (mouse_buttons.just_pressed(MouseButton::Left) && is_pointed)
            || touched_position_list.any(|t| wig.area.contains(t))
        {
            if let Ok(ref_value) = read_ron(&type_registry, wig.ron.clone()) {
                bds_event.write(BdsEvent { value: ref_value });
            }
            let db_name_opt = writing_query
                .get(ta_parent.parent())
                .map(|x| x.name.clone());
            go_event.write(ButtonIsPushed {
                writing_name: db_name_opt.unwrap_or_default(),
                text_area_name: ta.name.clone(),
            });
            for (p_entity, pending) in &pending_query {
                if pending.name == wig.waiter_name {
                    commands.entity(p_entity).remove::<Pending>();
                }
            }
            for s_entity in &selective_query {
                let pending = Pending {
                    name: "Went".to_string(),
                };
                commands.entity(s_entity).insert(pending);
            }
            commands.entity(target_entity).remove::<WaitInputGo>();
        }
    }
}
