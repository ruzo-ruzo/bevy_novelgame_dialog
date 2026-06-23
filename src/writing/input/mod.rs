mod go_selected;
pub(crate) mod params;
mod shift_selected;

pub(crate) use params::*;

use go_selected::*;
use shift_selected::*;

use super::*;
use crate::writing::extensions::*;
use crate::writing::window_controller::*;
use crate::writing::DialogBoxCamera;
use bevy::window::PrimaryWindow;

// ゲームパッド持ってないので全体的に挙動が未確認

pub(super) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, go_selected.in_set(PhaseSet::Fire))
            .add_systems(Update, shift_selected.in_set(PhaseSet::Fire));
    }
}
