#![allow(clippy::too_many_arguments)]
pub mod debug;
pub mod dialog_box;
mod read_script;
pub mod ui_templates;
mod utility;

pub mod prelude {
    pub use crate::{debug::*, dialog_box::*, ui_templates::*};
}
