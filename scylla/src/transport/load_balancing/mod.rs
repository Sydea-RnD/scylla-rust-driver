//! Load balancing configurations\
//! `Session` can use any load balancing policy which implements the `LoadBalancingPolicy` trait\
//! See [the book](https://rust-driver.docs.scylladb.com/stable/load-balancing/load-balancing.html) for more information

use super::{cluster::ClusterData, NodeRef};
use crate::routing::Token;
use scylla_cql::{errors::QueryError, frame::types};

use std::time::Duration;

mod default;
mod plan;
pub use default::{DefaultPolicy, DefaultPolicyBuilder, LatencyAwarenessBuilder};
pub use plan::Plan;

/// Represents info about statement that can be used by load balancing policies.
#[derive(Default, Clone, Debug)]
pub struct RoutingInfo<'a> {
    /// Requested consistency information allows to route queries to the appropriate
    /// datacenters. E.g. queries with a LOCAL_ONE consistency should be routed to the same
    /// datacenter.
    pub consistency: types::Consistency,
    pub serial_consistency: Option<types::SerialConsistency>,

    /// Information about token and keyspace is the basis of token-aware routing.
    pub token: Option<Token>,
    pub keyspace: Option<&'a str>,

    /// If, while preparing, we received from the cluster information that the statement is an LWT,
    /// then we can use this information for routing optimisation. Namely, an optimisation
    /// can be performed: the query should be routed to the replicas in a predefined order
    /// (i. e. always try first to contact replica A, then B if it fails, then C, etc.).
    /// If false, the query should be routed normally.
    /// Note: this a Scylla-specific optimisation. Therefore, the flag will be always false for Cassandra.
    pub is_confirmed_lwt: bool,
}

/// The fallback list of nodes in the query plan.
///
/// It is computed on-demand, only if querying the most preferred node fails
/// (or when speculative execution is triggered).
pub type FallbackPlan<'a> = Box<dyn Iterator<Item = NodeRef<'a>> + Send + Sync + 'a>;

/// Policy that decides which nodes to contact for each query.
///
/// When a query is prepared to be sent to ScyllaDB/Cassandra, a `LoadBalancingPolicy`
/// implementation constructs a load balancing plan. That plan is a list of nodes to which
/// the driver will try to send the query. The first elements of the plan are the nodes which are
/// the best to contact (e.g. they might have the lowest latency).
///
/// Most queries are send on the first try, so the query execution layer rarely needs to know more
/// than one node from plan. To better optimize that case, `LoadBalancingPolicy` has two methods:
/// `pick` and `fallback`. `pick` returns a first node to contact for a given query, `fallback`
/// returns the rest of the load balancing plan.
///
/// `fallback` is called only after a failed send to `pick`ed node (or when executing
/// speculatively).
/// If a `pick` returns `None`, `fallback` will not be called.
///
/// Usually the driver needs only the first node from load balancing plan (most queries are send
/// successfully, and there is no need to retry).
///
/// This trait is used to produce an iterator of nodes to contact for a given query.
pub trait LoadBalancingPolicy: Send + Sync + std::fmt::Debug {
    /// Returns the first node to contact for a given query.
    fn pick<'a>(&'a self, query: &'a RoutingInfo, cluster: &'a ClusterData) -> Option<NodeRef<'a>>;

    /// Returns all contact-appropriate nodes for a given query.
    fn fallback<'a>(&'a self, query: &'a RoutingInfo, cluster: &'a ClusterData)
        -> FallbackPlan<'a>;

    /// Invoked each time a query succeeds.
    fn on_query_success(&self, _query: &RoutingInfo, _latency: Duration, _node: NodeRef<'_>) {}

    /// Invoked each time a query fails.
    fn on_query_failure(
        &self,
        _query: &RoutingInfo,
        _latency: Duration,
        _node: NodeRef<'_>,
        _error: &QueryError,
    ) {
    }

    /// Returns the name of load balancing policy.
    fn name(&self) -> String;
}
