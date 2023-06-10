use bevy::prelude::*;

use super::super::*;

#[derive(Component, Debug)]
pub struct Wiping {
    wipe_per_sec: f32,
}

pub fn trigger_type_animation(
    mut commands: Commands,
    mut untriggered: Query<(
        Entity,
        &mut TypingTimer,
        &mut Transform,
        &WritingStyle,
        &mut Visibility,
    )>,
    time: Res<Time>,
) {
    for (entity, mut timer, mut tf, w_style, mut visibility) in &mut untriggered {
        if timer.timer.tick(time.delta()).finished() {
            match w_style {
                WritingStyle::Wipe { sec: s } => {
                    tf.scale = Vec3::new(0., 1., 1.);
                    commands.entity(entity).insert(Wiping {
                        wipe_per_sec: 1.0 / s,
                    });
                }
                WritingStyle::Put => (),
            }
            *visibility = Visibility::Inherited;
            commands.entity(entity).remove::<TypingTimer>();
        }
    }
}

pub fn text_wipe(
    mut commands: Commands,
    mut target: Query<(Entity, &mut Wiping, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, wiping, mut tf) in &mut target {
        tf.scale.x += time.delta_seconds() * wiping.wipe_per_sec;
        if tf.scale.x > 1. {
            tf.scale.x = 1.0;
            commands.entity(entity).remove::<Wiping>();
        }
    }
}
