#![allow(clippy::too_many_arguments)]
pub mod debug;
mod read_script;
pub mod ui_templates;
mod utility;
pub mod writing;

pub mod prelude {
    pub use crate::debug::*;
    pub use crate::ui_templates::*;
    pub use crate::writing::*;
}
