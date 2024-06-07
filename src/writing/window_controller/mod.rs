use bevy::{prelude::*, sprite::Anchor};

pub mod choice;
pub mod popup;
pub mod sinkdown;
pub mod waiting;

use super::setup::SetupConfig;
use crate::read_script::*;
use crate::writing::settings::configs::*;
use crate::writing::settings::params::*;
use crate::writing::OpenDialog;

#[derive(Component)]
pub(crate) struct DialogBox {
    pub name: String,
}

#[derive(Component)]
pub(crate) struct TextArea {
    pub name: String,
}

#[allow(private_interfaces)]
#[derive(Component)]
pub(crate) struct Current;

#[allow(private_interfaces)]
#[derive(Component)]
pub(crate) struct Pending;

#[derive(Component)]
pub(in crate::writing) struct Instant;

#[derive(Bundle)]
struct DialogBoxBundle {
    writing: DialogBox,
    state: DialogBoxPhase,
    waitting: WaitBrakerStyle,
    script: LoadedScript,
    popup_type: PopupType,
}

#[derive(Bundle)]
struct TextAreaBundle {
    text_area: TextArea,
    feeding: FeedingStyle,
    config: TypeTextConfig,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) enum DialogBoxPhase {
    Preparing,
    PoppingUp,
    WaitToType,
    Typing,
    WaitingAction,
    Feeding,
    Fixed,
    SinkingDown,
}
