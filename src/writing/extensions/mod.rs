pub mod bds;
pub(crate) mod params;

pub use bds::*;
pub(crate) use params::*;

use super::*;

pub(super) struct ExtensionsPlugin;

impl Plugin for ExtensionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<OpenDialog>()
            .add_message::<ButtonIsSelected>()
            .add_message::<ButtonIsPushed>()
            .add_message::<FinisClosingBox>()
            .add_message::<BdsSignal>()
            .add_message::<BdsEvent>()
            .add_systems(
                Update,
                change_current_text_area_in_current_box.in_set(PhaseSet::Setting),
            )
            .add_systems(Update, change_font_size.in_set(PhaseSet::Setting))
            .add_systems(Update, change_current_text_area.in_set(PhaseSet::Setting))
            .add_systems(Update, change_current_writing.in_set(PhaseSet::Setting))
            .add_systems(Update, load_bds.in_set(PhaseSet::Fire))
            .add_systems(Update, send_bds_signal.in_set(PhaseSet::Fire))
            .add_systems(Update, force_feeding_current_box.in_set(PhaseSet::Fire));
    }
}
