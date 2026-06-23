use crate::read_script::*;
use bevy::prelude::*;

pub mod extensions;

pub(crate) mod input;
pub(crate) mod settings;
pub(crate) mod window_controller;

mod setup;
mod text_controller;

pub use extensions::*;
pub use settings::configs::*;

pub(crate) use settings::params::*;

use input::*;
use setup::*;
use text_controller::*;
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
// SettingはProgressの挙動を変えかねない設定（CurrentやDialogBoxPhase）を変更します。
// ProgressはEventを受け取って実行します。
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhaseSet {
    Fire,
    Setting,
    Progress,
}

impl Plugin for DialogBoxPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (PhaseSet::Fire, PhaseSet::Setting, PhaseSet::Progress).chain(),
        )
        .init_asset::<BMWScript>()
        .init_asset::<BMWTemplate>()
        .init_asset_loader::<BMWScriptLoader>()
        .init_asset_loader::<BMWTemplateLoader>()
        .insert_resource(SetupConfig {
            render_layer: self.layer_num,
            render_order: self.render_order,
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Update, script_on_load.in_set(PhaseSet::Setting))
        .add_plugins((
            TypeTextPlugin,
            WindowControllerPlugin,
            ExtensionsPlugin,
            InputPlugin,
        ));
    }
}
