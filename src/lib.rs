#![allow(clippy::too_many_arguments)]
pub mod debug;
pub mod dialog_box;
mod read_script;
pub mod ui_templates;
mod utility;
pub use crate::dialog_box::public as preference;

pub mod prelude {
    pub use crate::debug::*;
    pub use crate::dialog_box::*;
    pub use crate::ui_templates::*;
}
