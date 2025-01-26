//! Reusable change selector UI for source control systems.

#![warn(missing_docs)]
#![warn(
    clippy::all,
    clippy::as_conversions,
    clippy::clone_on_ref_ptr,
    clippy::dbg_macro
)]
#![allow(clippy::too_many_arguments, clippy::blocks_in_conditions)]

mod render;
mod types;
mod ui;
mod util;

pub mod consts;
pub mod helpers;
pub use types::{
    ChangeType, Commit, File, FileState, FileMode, RecordError, RecordState, Section, SectionChangedLine,
    SelectedContents,
};
pub use ui::{Event, RecordInput, Recorder, TerminalKind, TestingScreenshot};
