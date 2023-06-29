use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::read_script::*;
use crate::message_window::MessageWindowCamera;

#[derive(Component, Debug)]
pub struct Selected;

#[derive(Component, Debug)]
pub struct WaitInputGo {
    pub ron: String,
    pub area: Rect,
}

#[allow(clippy::nonminimal_bool)]
pub fn go_selected(
    mut commands: Commands,
    target_query: Query<(Entity, &WaitInputGo)>,
    selected_query: Query<Entity, With<Selected>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MessageWindowCamera>>,
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    gamepad_buttons: Res<Input<GamepadButton>>,
    mut bms_event: EventWriter<BMSEvent>,
    gamepads: Res<Gamepads>,
    type_registry: Res<AppTypeRegistry>,
){
    for (target_entity, wig) in &target_query {
        let pointed_opt  = camera_query.get_single().ok()
            .and_then(|x|window_query.get_single().ok().and_then(|y|y.cursor_position()).map(|y|(x, y)))
            .and_then(|(c, p)| c.0.viewport_to_world_2d(c.1, p));
        let mut touched_position_list = touches.iter_just_pressed()
            .filter_map(|t| camera_query.get_single().ok().map(|c|(c, t)))
            .filter_map(|(c, t)| c.0.viewport_to_world_2d(c.1, t.position()));
        let is_selected = selected_query.get_single().is_ok_and(|e| e == target_entity);
        let is_pointed = pointed_opt.is_some_and(|x|(wig.area.contains(x)));
        let gamepad_go_button = gamepads.iter().next().map(|x|GamepadButton {
            gamepad: x,
            button_type: GamepadButtonType::South
        });
        // info!("area: {:?}, point: {:?}, pointed: {}",wig.area, pointed_opt, is_pointed);
        if (keys.any_just_pressed([KeyCode::Space, KeyCode::Return]) && is_selected) ||
            (gamepad_go_button.is_some_and(|x|gamepad_buttons.just_pressed(x)) && is_selected) ||
            (mouse_buttons.just_pressed(MouseButton::Left) && is_pointed) ||
            touched_position_list.any(|t| wig.area.contains(t))
        {
            if let Ok(ref_value) = read_ron(&type_registry, wig.ron.clone()) {
                bms_event.send(BMSEvent{ value:  ref_value});
            }
            commands.entity(target_entity).remove::<WaitInputGo>();
        }
    }
}
