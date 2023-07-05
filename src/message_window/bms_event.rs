use bevy::prelude::*;

use crate::read_script::BMSEvent;
use crate::*;

#[derive(Reflect, Default, Debug)]
pub struct FontSizeChange {
    pub size: f32,
}

pub fn change_font_size(
    mut events: EventReader<BMSEvent>,
    mut tb_query: Query<&mut TypeTextConfig, (With<Current>, With<TextBox>)>,
) {
    for event_wrapper in events.iter() {
        if let Some(FontSizeChange { size: s }) = event_wrapper.get_opt::<FontSizeChange>() {
            if let Ok(mut config) = tb_query.get_single_mut() {
                // info!("change font size to {}", s);
                config.text_style.font_size = s;
            }
        }
    }
}
