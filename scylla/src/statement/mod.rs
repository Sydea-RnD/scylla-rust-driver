use std::{sync::Arc, time::Duration};

use crate::history::HistoryListener;
use crate::transport::execution_profile::ExecutionProfileHandle;

pub mod batch;
pub mod prepared_statement;
pub mod query;

pub use crate::frame::types::{Consistency, SerialConsistency};

#[derive(Debug)]
pub struct StatementConfig {
    pub consistency: Option<Consistency>,
    pub serial_consistency: Option<Option<SerialConsistency>>,

    pub is_idempotent: bool,

    pub tracing: bool,
    pub timestamp: Option<i64>,
    pub request_timeout: Option<Duration>,

    pub history_listener: Option<Arc<dyn HistoryListener>>,

    pub execution_profile_handle: Option<ExecutionProfileHandle>,
}

#[allow(clippy::derivable_impls)]
impl Default for StatementConfig {
    fn default() -> Self {
        Self {
            consistency: Default::default(),
            serial_consistency: None,
            is_idempotent: false,
            tracing: false,
            timestamp: None,
            request_timeout: None,
            history_listener: None,
            execution_profile_handle: None,
        }
    }
}

impl Clone for StatementConfig {
    fn clone(&self) -> Self {
        Self {
            history_listener: self.history_listener.clone(),
            execution_profile_handle: self.execution_profile_handle.clone(),
            ..*self
        }
    }
}

impl StatementConfig {
    /// Determines the consistency of a query
    #[must_use]
    pub fn determine_consistency(&self, default_consistency: Consistency) -> Consistency {
        self.consistency.unwrap_or(default_consistency)
    }
}
