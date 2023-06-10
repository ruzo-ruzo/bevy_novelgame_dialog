use super::*;
use bevy::render::view::Visibility::Visible;

#[derive(Component, Debug)]
pub struct ScalingUp {
    pub add_per_sec: f32,
}

pub fn window_popper(
    mut commands: Commands,
    mut mw_query: Query<
        (
            Entity,
            &mut WindowState,
            &PopupType,
            &mut Visibility,
            &mut Transform,
        ),
        With<MessageWindow>,
    >,
) {
    for (ent, mut ws, pt, mut vis, mut tf) in &mut mw_query {
        if *ws == WindowState::Preparing {
            match pt {
                PopupType::Scale { sec: s } => {
                    tf.scale = Vec3::new(0., 0., 0.);
                    commands.entity(ent).insert(ScalingUp {
                        add_per_sec: 1.0 / s,
                    });
                }
            }
            *vis = Visible;
            *ws = WindowState::PoppingUp;
        }
    }
}

pub fn scaling_up(
    mut commands: Commands,
    mut mw_query: Query<(Entity, &mut Transform, &ScalingUp, &mut WindowState)>,
    time: Res<Time>,
) {
    for (ent, mut tf, ScalingUp { add_per_sec: aps }, mut ws) in &mut mw_query {
        if tf.scale.x >= 1.0 {
            tf.scale = Vec3::new(1., 1., 1.);
            *ws = WindowState::Typing;
            commands.entity(ent).remove::<ScalingUp>();
        } else {
            tf.scale.x += time.delta_seconds() * aps;
            tf.scale.y += time.delta_seconds() * aps;
        };
    }
}
