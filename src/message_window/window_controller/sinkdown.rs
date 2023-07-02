use super::*;
use crate::message_window::*;

#[derive(Component, Debug)]
pub struct ScalingDown {
    pub sub_per_sec: f32,
}

#[derive(Reflect, Default, Debug)]
pub struct SinkDownWindow {
    pub sink_type: SinkDownType,
}

#[derive(Component, Debug)]
pub struct WaitSinkingTrigger {
    pub sink_type: SinkDownType,
    pub timer: Timer,
}

#[derive(Reflect, Default, Event)]
pub struct GoSinking {
    pub target: Option<Entity>,
    pub sink_type: SinkDownType,
}

#[allow(clippy::type_complexity)]
pub fn setup_window_sink(
    mut commands: Commands,
    text_query: Query<(Entity, &TypingTimer), (With<Current>, With<MessageTextChar>)>,
    text_box_query: Query<(Entity, &GlobalTransform, &Sprite), (With<Current>, With<TextBox>)>,
    mut mw_query: Query<(Entity, &mut WindowState, &WaitBrakerStyle), (With<Current>, With<MessageWindow>)>,
    parents: Query<&Parent>,
    mut events: EventReader<BMSEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event_wrapper in events.iter() {
        if let Some(SinkDownWindow {
            sink_type: sdt,
        }) = event_wrapper.get_opt::<SinkDownWindow>()
        {
            for (mw_entity, mut ws, wbs) in &mut mw_query {
                match wbs {
                    WaitBrakerStyle::Auto { wait_sec: base_sec } => {
                        let count: f32 = text_query.iter()
                            .filter(|(tx_entity, _)|parents.iter_ancestors(*tx_entity).any(|x| x == mw_entity) )
                            .map(|(_, tt)|tt.timer.remaining_secs())
                            .sum();
                        commands.entity(mw_entity).insert(WaitSinkingTrigger {
                            sink_type: sdt,
                            timer: Timer::from_seconds(
                                base_sec + count,
                                TimerMode::Once,
                            ),
                        });
                    },
                    WaitBrakerStyle::Input {..} => {
                        if let Some((target_tb, tb_tf,  tb_sp)) = text_box_query.iter()
                            .find(|(tb_e, ..)|parents.iter_ancestors(*tb_e).any(|pa_e|pa_e == mw_entity))
                        {
                            let gs_ron = write_ron(&type_registry, GoSinking { target: Some(mw_entity),  sink_type: sdt });
                            let wig =  make_wig(target_tb, tb_tf,  tb_sp, gs_ron.unwrap_or_default(), &type_registry);
                            commands.entity(target_tb).insert(wig);
                        }
                    },
                }
                *ws = WindowState::Waiting
            }
        }
    }
}

pub fn trigger_window_sink_by_event(
    mut bms_reader: EventReader<BMSEvent>,
    mut gs_writer: EventWriter<GoSinking>,
){
    for event_wrapper in bms_reader.iter() {
        if let Some(gs @ GoSinking {..}) = event_wrapper.get_opt::<GoSinking>() {
            gs_writer.send(gs)
        }
    }
}

pub fn trigger_window_sink_by_time(
    mut commands: Commands,
    mut mw_query: Query<(Entity, &mut WaitSinkingTrigger), With<MessageWindow>>,
    time: Res<Time>,
    mut events: EventWriter<GoSinking>,
){
    for (entity,mut wst) in &mut mw_query {
        if wst.timer.tick(time.delta()).finished() {
            events.send(GoSinking {
                target: Some(entity),
                sink_type: wst.sink_type,
            });
            commands.entity(entity).remove::<WaitSinkingTrigger>();
        }
    }
}

pub fn start_window_sink(
    mut commands: Commands,
    mut mw_query: Query<(Entity, &mut WindowState), With<MessageWindow>>,
    mut events: EventReader<GoSinking>,
) {
    for GoSinking{target: entity_opt, sink_type: st} in &mut events {
        info!("go sink");
        for (mw_entity, mut ws) in &mut mw_query {
            if let Some(entity) = *entity_opt {
                if entity == mw_entity {
                    match st {
                        SinkDownType::Scale { sec: s } => {
                            commands.entity(entity).insert(ScalingDown {
                                sub_per_sec: 1.0 / s,
                            });
                            *ws = WindowState::SinkingDown;
                        }
                        SinkDownType::Fix => *ws = WindowState::Fixed,
                    }
                }
            }
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
