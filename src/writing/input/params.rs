use super::*;

#[derive(Component, Debug)]
pub(crate) struct Selected;

#[derive(Component, Debug)]
pub(in crate::writing) struct WaitInputGo {
    pub ron: String,
    pub area: Rect,
    pub waiter_name: String,
}

#[derive(Component)]
pub(in crate::writing) struct Selective {
    pub key_vector: SelectVector,
    pub number: usize,
}
