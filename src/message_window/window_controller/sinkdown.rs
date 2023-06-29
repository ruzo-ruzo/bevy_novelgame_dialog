use super::*;
use crate::message_window::*;

#[derive(Component, Debug)]
pub struct ScalingDown {
    pub sub_per_sec: f32,
}

#[derive(Reflect, Default, Debug)]
pub struct SinkDownWindow {
    pub sink_type: SinkDownType,
    pub wait_sec: f32,
}

#[derive(Component, Debug)]
pub struct WaitSinkingTrigger {
    pub sink_type: SinkDownType,
    pub timer: Timer,
}

#[allow(clippy::type_complexity)]
pub fn setup_window_sinker(
    mut commands: Commands,
    text_query: Query<(Entity, &TypingTimer), (With<Current>, With<MessageTextChar>)>,
    mut mw_query: Query<(Entity, &mut WindowState), (With<Current>, With<MessageWindow>)>,
    parents: Query<&Parent>,
    mut events: EventReader<BMSEvent>,
) {
    for event_wrapper in events.iter() {
        if let Ok((tx_entity, tt)) = text_query.get_single() {
            for (mw_entity, mut ws) in &mut mw_query {
                if parents.iter_ancestors(tx_entity).any(|x| x == mw_entity) {
                    if let Some(SinkDownWindow {
                        sink_type: sdt,
                        wait_sec: sec,
                    }) = event_wrapper.get_opt::<SinkDownWindow>()
                    {
                        commands.entity(mw_entity).insert(WaitSinkingTrigger {
                            sink_type: sdt,
                            timer: Timer::from_seconds(
                                sec + tt.timer.remaining_secs(),
                                TimerMode::Once,
                            ),
                        });
                        *ws = WindowState::SinkingDown
                    }
                }
            }
        }
    }
}

pub fn window_sinker(
    mut commands: Commands,
    mut mw_query: Query<(Entity, &mut WindowState, &mut WaitSinkingTrigger)>,
    time: Res<Time>,
) {
    for (entity, mut ws, mut wst) in &mut mw_query {
        if wst.timer.tick(time.delta()).finished() {
            match wst.sink_type {
                SinkDownType::Scale { sec: s } => {
                    commands.entity(entity).insert(ScalingDown {
                        sub_per_sec: 1.0 / s,
                    });
                }
                SinkDownType::Fix => *ws = WindowState::Fixed,
            };
            commands.entity(entity).remove::<WaitSinkingTrigger>();
        }
    }
}

pub fn scaling_down(
    mut commands: Commands,
    mut mw_query: Query<(Entity, &mut Transform, &ScalingDown)>,
    time: Res<Time>,
) {
    for (entity, mut tf, ScalingDown { sub_per_sec: aps }) in &mut mw_query {
        if tf.scale.x <= 0.0 {
            tf.scale = Vec3::new(0., 0., 0.);
            commands.entity(entity).despawn_recursive();
        } else {
            tf.scale.x -= time.delta_seconds() * aps;
            tf.scale.y -= time.delta_seconds() * aps;
        };
    }
}
