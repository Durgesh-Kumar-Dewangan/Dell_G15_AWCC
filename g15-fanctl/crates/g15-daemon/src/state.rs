use g15_common::dell_iface::DellInterface;
use g15_common::sensors::{CpuUtilTracker, Snapshot};
use g15_common::settings::PersistedState;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// All mutable daemon state, behind a single RwLock so the D-Bus handlers and the
/// background polling loop never race each other.
pub struct DaemonState {
    pub dell: DellInterface,
    pub cpu_tracker: CpuUtilTracker,
    pub last_snapshot: Snapshot,
    pub persisted: PersistedState,
    pub state_path: PathBuf,
}

pub type SharedState = Arc<RwLock<DaemonState>>;

impl DaemonState {
    pub fn new_shared() -> anyhow::Result<SharedState> {
        let dell = DellInterface::probe()?;
        let state_path = PersistedState::default_system_path();
        let persisted = PersistedState::load(&state_path)?;
        Ok(Arc::new(RwLock::new(DaemonState {
            dell,
            cpu_tracker: CpuUtilTracker::new(),
            last_snapshot: Snapshot::default(),
            persisted,
            state_path,
        })))
    }
}
