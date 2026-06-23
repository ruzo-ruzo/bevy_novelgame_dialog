use bevy::{prelude::*, sprite::Anchor};

pub mod choice;
pub mod params;
pub mod popup;
pub mod sinkdown;
pub mod waiting;

pub(crate) use params::*;

use super::*;
use choice::*;
use popup::*;
use sinkdown::*;
use waiting::*;

use crate::writing::settings::configs::*;
use crate::writing::settings::params::*;
use crate::writing::setup::SetupConfig;
use crate::writing::OpenDialog;

pub(super) struct WindowControllerPlugin;

impl Plugin for WindowControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GoSinking>()
            .add_systems(Update, setup_window_sink.in_set(PhaseSet::Setting))
            .add_systems(Update, despawn_writing.in_set(PhaseSet::Setting))
            .add_systems(Update, remove_pending.in_set(PhaseSet::Setting))
            .add_systems(Update, setup_choice.in_set(PhaseSet::Setting))
            .add_systems(
                Update,
                reinstatement_external_entities.in_set(PhaseSet::Setting),
            )
            .add_systems(Update, restart_typing.in_set(PhaseSet::Setting))
            .add_systems(Update, waiting_icon_setting.in_set(PhaseSet::Setting))
            .add_systems(Update, scaling_up.in_set(PhaseSet::Progress))
            .add_systems(Update, scaling_down.in_set(PhaseSet::Progress))
            .add_systems(Update, simple_wait.in_set(PhaseSet::Progress))
            .add_systems(Update, settle_wating_icon.in_set(PhaseSet::Progress))
            .add_systems(Update, skip_typing_or_next.in_set(PhaseSet::Progress))
            .add_systems(Update, hide_waiting_icon.in_set(PhaseSet::Progress))
            .add_systems(Update, open_window.in_set(PhaseSet::Fire))
            .add_systems(Update, window_popper.in_set(PhaseSet::Fire))
            .add_systems(Update, trigger_window_sink_by_event.in_set(PhaseSet::Fire))
            .add_systems(Update, start_window_sink.in_set(PhaseSet::Fire))
            .add_systems(Update, trigger_window_sink_by_time.in_set(PhaseSet::Fire))
            .add_systems(Update, open_choice_box.in_set(PhaseSet::Fire))
            .add_systems(Update, close_choice_phase.in_set(PhaseSet::Fire))
            .add_systems(
                Update,
                skip_feeding.in_set(PhaseSet::Fire).after(add_new_text),
            );
    }
}
