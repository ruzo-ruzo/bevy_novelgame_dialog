use crate::read_script::*;
use bevy::prelude::*;

pub mod events;
pub(crate) mod input;
pub mod settings;
mod setup;
mod text_controller;
pub(crate) mod window_controller;

pub use events::*;
use input::*;
pub use settings::configs::*;
pub use settings::params::*;
use setup::*;
use text_controller::feed_animation::*;
use text_controller::typing_animations::*;
use text_controller::*;
use window_controller::choice::*;
use window_controller::popup::*;
use window_controller::sinkdown::*;
use window_controller::waiting::*;
use window_controller::*;

pub struct DialogBoxPlugin {
    pub layer_num: u8,
    pub render_order: isize,
}

impl Default for DialogBoxPlugin {
    fn default() -> Self {
        DialogBoxPlugin {
            layer_num: 2,
            render_order: 1,
        }
    }
}

// FireはEventを発行します。
// ProgressはEventを受け取って実行します。
// SettingはProgressの挙動を変えかねない設定（CurrentやDialogBoxPhase）を変更します。
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhaseSet {
    Setting,
    Progress,
    Fire,
}

impl Plugin for DialogBoxPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BMWScript>()
            .init_asset::<BMWTemplate>()
            .init_asset_loader::<BMWScriptLoader>()
            .init_asset_loader::<BMWTemplateLoader>()
            .insert_resource(SetupConfig {
                render_layer: self.layer_num,
                render_order: self.render_order,
            })
            .register_type::<ChangeFontSize>()
            .register_type::<ChangeCurrentTextArea>()
            .register_type::<ChangeCurrentTextAreaInCurrentBox>()
            .register_type::<ChangeCurrentDialogBox>()
            .register_type::<LoadBds>()
            .register_type::<(String, String)>()
            .register_type::<Vec<(String, String)>>()
            .register_type::<SetupChoice>()
            .register_type::<ChoosenEvent>()
            .register_type::<SinkDownWindow>()
            .register_type::<InputForFeeding>()
            .register_type::<InputForSkipping>()
            .register_type::<GoSinking>()
            .register_type::<SinkDownType>()
            .register_type::<SimpleWait>()
            .register_type::<BreakWait>()
            .register_type::<SimpleStringSignal>()
            .register_type::<ForceFeedingCurrentBox>()
            .add_event::<OpenDialog>()
            .add_event::<FeedWaitingEvent>()
            .add_event::<StartFeedingEvent>()
            .add_event::<ButtonIsSelected>()
            .add_event::<ButtonIsPushed>()
            .add_event::<GoSinking>()
            .add_event::<FinisClosingBox>()
            .add_event::<BdsSignal>()
            .add_event::<BdsEvent>()
            .configure_sets(
                Update,
                (PhaseSet::Setting, PhaseSet::Progress, PhaseSet::Fire).chain(),
            )
            .add_systems(Startup, setup_camera)
            .add_systems(Update, script_on_load.in_set(PhaseSet::Setting))
            .add_systems(Update, trigger_type_animation.in_set(PhaseSet::Setting))
            .add_systems(Update, setup_window_sink.in_set(PhaseSet::Setting))
            .add_systems(Update, waiting_icon_setting.in_set(PhaseSet::Setting))
            .add_systems(Update, start_feeding.in_set(PhaseSet::Setting))
            .add_systems(Update, restart_typing.in_set(PhaseSet::Setting))
            .add_systems(Update, change_current_text_area.in_set(PhaseSet::Setting))
            .add_systems(
                Update,
                change_current_text_area_in_current_box.in_set(PhaseSet::Setting),
            )
            .add_systems(Update, change_current_writing.in_set(PhaseSet::Setting))
            .add_systems(Update, change_font_size.in_set(PhaseSet::Setting))
            .add_systems(Update, setup_choice.in_set(PhaseSet::Setting))
            .add_systems(Update, despawn_writing.in_set(PhaseSet::Setting))
            .add_systems(Update, remove_pending.in_set(PhaseSet::Setting))
            .add_systems(
                Update,
                reinstatement_external_entities.in_set(PhaseSet::Setting),
            )
            .add_systems(Update, add_new_text.in_set(PhaseSet::Fire))
            .add_systems(Update, settle_wating_icon.in_set(PhaseSet::Progress))
            .add_systems(Update, settle_lines.in_set(PhaseSet::Progress))
            .add_systems(Update, text_wipe.in_set(PhaseSet::Progress))
            .add_systems(Update, hide_waiting_icon.in_set(PhaseSet::Progress))
            .add_systems(Update, scaling_up.in_set(PhaseSet::Progress))
            .add_systems(Update, scaling_down.in_set(PhaseSet::Progress))
            .add_systems(Update, scroll_lines.in_set(PhaseSet::Progress))
            .add_systems(Update, simple_wait.in_set(PhaseSet::Progress))
            .add_systems(Update, skip_typing_or_next.in_set(PhaseSet::Progress))
            .add_systems(Update, setup_feed_starter.in_set(PhaseSet::Progress))
            .add_systems(Update, open_window.in_set(PhaseSet::Fire))
            .add_systems(Update, open_choice_box.in_set(PhaseSet::Fire))
            .add_systems(Update, load_bds.in_set(PhaseSet::Fire))
            .add_systems(Update, window_popper.in_set(PhaseSet::Fire))
            .add_systems(Update, start_window_sink.in_set(PhaseSet::Fire))
            .add_systems(Update, trigger_feeding_by_time.in_set(PhaseSet::Fire))
            .add_systems(Update, close_choice_phase.in_set(PhaseSet::Fire))
            .add_systems(Update, trigger_feeding_by_event.in_set(PhaseSet::Fire))
            .add_systems(Update, go_selected.in_set(PhaseSet::Fire))
            .add_systems(Update, shift_selected.in_set(PhaseSet::Fire))
            .add_systems(Update, force_feeding_current_box.in_set(PhaseSet::Fire))
            .add_systems(Update, send_bds_signal.in_set(PhaseSet::Fire))
            .add_systems(
                Update,
                skip_feeding.in_set(PhaseSet::Fire).after(add_new_text),
            )
            .add_systems(Update, trigger_window_sink_by_time.in_set(PhaseSet::Fire))
            .add_systems(Update, trigger_window_sink_by_event.in_set(PhaseSet::Fire));
    }
}
