use bevy::prelude::*;

use super::super::*;

#[derive(Component, Debug)]
pub(in crate::dialog_box) enum TypingStyle {
    Wiping { wipe_per_sec: f32 },
    Typed,
}

pub(in crate::dialog_box) fn trigger_type_animation(
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
        if timer.timer.tick(time.delta()).finished() {
            match w_style {
                WritingStyle::Wipe { sec: s } => {
                    tf.scale = Vec3::new(0., 1., 1.);
                    commands.entity(entity).insert(TypingStyle::Wiping {
                        wipe_per_sec: 1.0 / s,
                    });
                }
                WritingStyle::Put => (),
            }
            *visibility = Visibility::Inherited;
        }
    }
}

pub(in crate::dialog_box) fn text_wipe(
    mut commands: Commands,
    mut target: Query<(Entity, &TypingStyle, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, ts, mut tf) in &mut target {
        if let TypingStyle::Wiping { wipe_per_sec: sec } = ts {
            tf.scale.x += time.delta_seconds() * sec;
            if tf.scale.x > 1. {
                tf.scale.x = 1.0;
                commands.entity(entity).remove::<TypingStyle>();
                commands.entity(entity).insert(TypingStyle::Typed);
            }
        }
    }
}
