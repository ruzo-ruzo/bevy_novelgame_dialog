use bevy::{
    prelude::*,
    sprite::Anchor,
};

pub mod choice;
pub mod popup;
pub mod sinkdown;
pub mod waiting;

use super::setup::SetupConfig;
use crate::dialog_box::OpenDialogEvent;
use crate::read_script::*;
use crate::dialog_box::public::configs::*;

#[derive(Component, Debug)]
pub struct DialogBox {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct TextArea {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct Current;

#[derive(Component)]
pub struct Pending;

#[derive(Component)]
pub struct Instant;

#[derive(Bundle)]
struct DialogBoxBundle {
    dialog_box: DialogBox,
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
pub enum DialogBoxPhase {
    Preparing,
    PoppingUp,
    WaitToType,
    Typing,
    WaitingAction,
    Feeding,
    Fixed,
    SinkingDown,
}

// ComponentにEntityつっこむのヤバいので後で直す
// 名前の重複を防ぐ機構を入れた方がいいかもしれない
#[derive(Component)]
pub struct ChoiceBoxState {
    main_dialog_box: Entity,
    choice_box_entity: Option<Entity>,
    text_area_names: Vec<String>,
    button_entities: Vec<Entity>,
    target_list: Vec<(String, String)>,
    select_vector: SelectVector,
    sinkdown: SinkDownType,
    background_scaling_per_button: Vec2,
    background_scaling_anchor: Anchor,
}
