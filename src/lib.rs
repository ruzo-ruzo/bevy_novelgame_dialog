#![allow(clippy::too_many_arguments)]
pub mod debug;
pub mod ui_templates;
pub mod writing;

pub mod prelude {
    pub use crate::debug::*;
    pub use crate::ui_templates::*;
    pub use crate::writing::*;
}

mod read_script;
mod utility;
