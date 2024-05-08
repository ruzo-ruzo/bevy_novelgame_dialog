use super::*;

// 倍数版欲しいかも
#[derive(Reflect, Default, Debug)]
pub struct ChangeFontSize {
    pub size: f32,
}

pub fn change_font_size(
    mut events: EventReader<BdsEvent>,
    mut ta_query: Query<&mut TypeTextConfig, (With<Current>, With<TextArea>)>,
) {
    for event_wrapper in events.read() {
        if let Some(ChangeFontSize { size: s }) = event_wrapper.get_opt::<ChangeFontSize>() {
            if let Ok(mut config) = ta_query.get_single_mut() {
                config.text_style.font_size = s;
            }
        }
    }
}
