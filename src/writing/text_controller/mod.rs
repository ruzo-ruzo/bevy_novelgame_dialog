use bevy::prelude::*;

pub(super) mod feed_animation;
pub(super) mod type_text;
pub(super) mod typing_animations;

use super::*;

pub(super) use feed_animation::*;
pub(super) use type_text::*;
pub(super) use typing_animations::*;

pub(super) struct TypeTextPlugin;

impl Plugin for TypeTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FeedWaitingEvent>()
            .add_message::<StartFeedingEvent>()
            .add_systems(Update, trigger_type_animation.in_set(PhaseSet::Setting))
            .add_systems(Update, settle_lines.in_set(PhaseSet::Progress))
            .add_systems(Update, setup_feed_starter.in_set(PhaseSet::Progress))
            .add_systems(Update, text_wipe.in_set(PhaseSet::Progress))
            .add_systems(Update, start_feeding.in_set(PhaseSet::Setting))
            .add_systems(Update, scroll_lines.in_set(PhaseSet::Progress))
            .add_systems(Update, add_new_text.in_set(PhaseSet::Fire))
            .add_systems(Update, trigger_feeding_by_event.in_set(PhaseSet::Fire))
            .add_systems(Update, trigger_feeding_by_time.in_set(PhaseSet::Fire));
    }
}
