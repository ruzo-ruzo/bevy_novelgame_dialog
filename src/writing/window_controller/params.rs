use bevy::prelude::*;

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
pub(crate) struct Pending {
    pub name: String,
}

#[derive(Component)]
pub(in crate::writing) struct Instant;

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

pub(crate) struct MakeWigConfig<'a, S: AsRef<str>> {
    pub dialog_box_name: S,
    pub text_area_name: S,
    pub waiter_name: S,
    pub ron: S,
    pub type_registry: &'a AppTypeRegistry,
}
