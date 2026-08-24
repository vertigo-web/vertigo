//! One module per app shape. Each owns its state struct, its builder and its render
//! function; the workloads in `crate::workloads` drive them.

pub mod dash;
pub mod editor;
pub mod list;
pub mod probe;
