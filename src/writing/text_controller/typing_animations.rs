use crate::writing::text_controller::*;
use crate::writing::*;

#[derive(Component, Debug)]
pub(in crate::writing) enum TypingStyle {
    Wiping { wipe_per_sec: f32 },
    Typed,
}

pub(super) fn trigger_type_animation(
    mut commands: Commands,
    mut untriggered: Query<
        (
            Entity,
            &mut TypingTimer,
            &mut Transform,
            &WritingStyle,
            &mut Visibility,
        ),
        Without<TypingStyle>,
    >,
    time: Res<Time>,
) {
    for (entity, mut timer, mut tf, w_style, mut visibility) in &mut untriggered {
        if timer.timer.tick(time.delta()).is_finished() {
            match w_style {
                WritingStyle::Wipe { sec: s } => {
                    tf.scale = Vec3::new(0.0, 1.0, 1.0);
                    commands.entity(entity).insert(TypingStyle::Wiping {
                        wipe_per_sec: 1.0 / s,
                    });
                }
                WritingStyle::Put => {
                    commands.entity(entity).insert(TypingStyle::Typed);
                }
            }
            *visibility = Visibility::Inherited;
        }
    }
}

pub(super) fn text_wipe(
    mut commands: Commands,
    mut target: Query<(Entity, &TypingStyle, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, ts, mut tf) in &mut target {
        if let TypingStyle::Wiping { wipe_per_sec: sec } = ts {
            tf.scale.x += time.delta_secs() * sec;
            if tf.scale.x > 1. {
                tf.scale.x = 1.0;
                commands.entity(entity).insert(TypingStyle::Typed);
            }
        }
    }
}
