#![allow(clippy::too_many_arguments)]
pub mod debug;
pub mod dialog_box;
mod read_script;
pub mod ui_templates;
mod utility;
pub use crate::dialog_box as core;

pub mod prelude {
    pub use crate::dialog_box::*;
    pub use crate::ui_templates::*;
    pub use crate::debug::*;
}
