mod matcher;
mod report;
mod snapshot;
mod target_tree;

pub(crate) use matcher::{Reconciled, discard, reconcile};
pub(crate) use snapshot::DomSnapshot;
#[cfg(test)]
pub(crate) use snapshot::{SnapshotAttr, SnapshotNode};
pub(crate) use target_tree::split_buffer;
