use bevy::ecs::component::Component;

#[derive(Component, Debug, PartialEq)]
pub enum MessageWindowState {
    NoWindow,
    PoppingUp,
    Vanishing,
    Waiting,
    Writing,
    Scrolling,
}

impl Default for MessageWindowState {
    fn default() -> MessageWindowState {
        Self::NoWindow
    }
}

#[derive(Component, Debug)]
pub struct MessageWindow;

#[derive(Component, Debug)]
pub struct CurrentMessageWindow;
