use crate::read_script::*;
use bevy::prelude::*;

mod bms_event;
mod input;
mod message_writer;
mod setup;
pub mod window_controller;

use bms_event::*;
use input::*;
use message_writer::feed_animation::*;
use message_writer::skip_typing::*;
use message_writer::typing_animations::*;
use message_writer::*;
use setup::*;
use window_controller::popup::*;
use window_controller::sinkdown::*;
use window_controller::*;

pub struct MessageWindowPlugin {
    pub layer_num: u8,
    pub render_order: isize,
}

impl Default for MessageWindowPlugin {
    fn default() -> Self {
        MessageWindowPlugin {
            layer_num: 2,
            render_order: 1,
        }
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhaseSet {
    Setting,
    Progress,
    Change,
}

impl Plugin for MessageWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<BMWScript>()
            .init_asset_loader::<BMWScriptLoader>()
            .insert_resource(SetupConfig {
                render_layer: self.layer_num,
                render_order: self.render_order,
            })
            .register_type::<FontSizeChange>()
            .register_type::<SinkDownWindow>()
            .register_type::<Option<Entity>>()
            .register_type::<InputForFeeding>()
            .register_type::<InputForSkipping>()
            .register_type::<SinkDownType>()
            .add_event::<OpenWindowEvent>()
            .add_event::<FeedWaitingEvent>()
            .add_event::<StartFeedingEvent>()
            .add_event::<BMSEvent>()
            .configure_sets(
                Update,
                (PhaseSet::Progress, PhaseSet::Setting, PhaseSet::Change).chain(),
            )
            .add_systems(Startup, setup_camera)
            .add_systems(Update, script_on_load.in_set(PhaseSet::Setting))
            .add_systems(Update, trigger_type_animation.in_set(PhaseSet::Setting))
            .add_systems(Update, setup_feed_starter.in_set(PhaseSet::Setting))
            .add_systems(Update, change_font_size.in_set(PhaseSet::Setting))
            .add_systems(Update, setup_window_sinker.in_set(PhaseSet::Setting))
            .add_systems(Update, skip_or_next.in_set(PhaseSet::Setting))
            .add_systems(Update, settle_lines.in_set(PhaseSet::Progress))
            .add_systems(Update, text_wipe.in_set(PhaseSet::Progress))
            .add_systems(Update, scaling_up.in_set(PhaseSet::Progress))
            .add_systems(Update, scaling_down.in_set(PhaseSet::Progress))
            .add_systems(Update, scroll_lines.in_set(PhaseSet::Progress))
            .add_systems(Update, open_window.in_set(PhaseSet::Change))
            .add_systems(Update, window_popper.in_set(PhaseSet::Change))
            .add_systems(Update, window_sinker.in_set(PhaseSet::Change))
            .add_systems(Update, add_new_text.in_set(PhaseSet::Change))
            .add_systems(Update, trigger_feeding_by_time.in_set(PhaseSet::Change))
            .add_systems(Update, trigger_feeding_by_event.in_set(PhaseSet::Change))
            .add_systems(Update, go_selected.in_set(PhaseSet::Change))
            .add_systems(Update, start_feeding.in_set(PhaseSet::Change));
    }
}
