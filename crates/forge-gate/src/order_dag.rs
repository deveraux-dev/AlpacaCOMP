//! Deterministic order-state transition gate.
//!
//! Ported from Nistam `gemma-s13::logit_mask::RagDag` (crucible-mask pattern):
//! any candidate order-state transition not explicitly witnessed in the DAG
//! is clamped to `ORDER_REJECT` before it can reach execution.

/// Sentinel value marking a rejected/illegal order transition.
pub const ORDER_REJECT: i32 = i32::MIN;

/// Maximum number of order states in the static DAG.
pub const MAX_DAG_NODES: usize = 32;

/// Maximum outward transitions per order state.
pub const MAX_EDGES_PER_NODE: usize = 8;

/// A witnessed order state and its legal next states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllowedTransition {
    /// Order-state identifier.
    pub state_id: u32,
    /// Audit/provenance hash for this state.
    pub audit_tag: u64,
    /// Number of valid outward transitions.
    pub edge_count: usize,
    /// Legal next order-state identifiers.
    pub next_state_ids: [u32; MAX_EDGES_PER_NODE],
}

impl AllowedTransition {
    /// Create a new allowed-transition node.
    pub const fn new(state_id: u32, audit_tag: u64, allowed_next: &[u32]) -> Self {
        let mut next_state_ids = [0u32; MAX_EDGES_PER_NODE];
        let mut edge_count = 0;
        let mut i = 0;
        while i < allowed_next.len() && i < MAX_EDGES_PER_NODE {
            next_state_ids[i] = allowed_next[i];
            edge_count += 1;
            i += 1;
        }
        Self {
            state_id,
            audit_tag,
            edge_count,
            next_state_ids,
        }
    }

    /// Check if a transition to `candidate_state_id` is witnessed.
    #[inline]
    pub fn allows_transition(&self, candidate_state_id: u32) -> bool {
        let mut i = 0;
        while i < self.edge_count {
            if self.next_state_ids[i] == candidate_state_id {
                return true;
            }
            i += 1;
        }
        false
    }
}

/// Static order-state transition DAG.
pub struct OrderStateDag {
    nodes: [AllowedTransition; MAX_DAG_NODES],
    node_count: usize,
}

impl OrderStateDag {
    /// Build a DAG from up to `MAX_DAG_NODES` allowed-transition nodes.
    pub const fn from_nodes(nodes_in: &[AllowedTransition]) -> Self {
        let mut nodes = [AllowedTransition {
            state_id: 0,
            audit_tag: 0,
            edge_count: 0,
            next_state_ids: [0; MAX_EDGES_PER_NODE],
        }; MAX_DAG_NODES];

        let mut i = 0;
        while i < nodes_in.len() && i < MAX_DAG_NODES {
            nodes[i] = nodes_in[i];
            i += 1;
        }
        let node_count = if nodes_in.len() < MAX_DAG_NODES {
            nodes_in.len()
        } else {
            MAX_DAG_NODES
        };

        Self { nodes, node_count }
    }

    /// Find node by order-state id.
    #[inline]
    pub fn find_node(&self, state_id: u32) -> Option<&AllowedTransition> {
        let mut i = 0;
        while i < self.node_count {
            if self.nodes[i].state_id == state_id {
                return Some(&self.nodes[i]);
            }
            i += 1;
        }
        None
    }

    /// Validate whether an entire multi-hop order-state sequence strictly
    /// follows witnessed edges in the DAG.
    #[inline]
    pub fn validate_path(&self, path: &[u32]) -> bool {
        if path.is_empty() {
            return false;
        }
        if path.len() == 1 {
            return self.find_node(path[0]).is_some();
        }
        let mut i = 0;
        while i + 1 < path.len() {
            let curr = path[i];
            let next = path[i + 1];
            match self.find_node(curr) {
                Some(node) => {
                    if !node.allows_transition(next) {
                        return false;
                    }
                }
                None => return false,
            }
            i += 1;
        }
        true
    }

    /// Mask all candidate order actions that are not witnessed as legal
    /// next states from `current_state_id` down to `ORDER_REJECT`.
    #[inline]
    pub fn apply_order_mask(&self, current_state_id: u32, candidate_scores: &mut [i32]) {
        if let Some(node) = self.find_node(current_state_id) {
            for (candidate_id, score) in candidate_scores.iter_mut().enumerate() {
                if !node.allows_transition(candidate_id as u32) {
                    *score = ORDER_REJECT;
                }
            }
        } else {
            // Un-witnessed current state: reject every candidate transition.
            for score in candidate_scores.iter_mut() {
                *score = ORDER_REJECT;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical 5-state order lifecycle DAG for tests:
    /// 0=Flat, 1=OpenLong, 2=OpenShort, 3=CloseLong, 4=CloseShort.
    /// Flat -> {OpenLong, OpenShort}; OpenLong -> {CloseLong}; OpenShort -> {CloseShort}.
    fn test_dag() -> OrderStateDag {
        OrderStateDag::from_nodes(&[
            AllowedTransition::new(0, 0xA001, &[1, 2]),
            AllowedTransition::new(1, 0xA002, &[3]),
            AllowedTransition::new(2, 0xA003, &[4]),
        ])
    }

    #[test]
    fn test_order_mask_allowed_transitions() {
        let dag = test_dag();
        let mut scores = [1000i32; 5];

        // Current state 0 (Flat). Allowed: 1 (OpenLong), 2 (OpenShort).
        dag.apply_order_mask(0, &mut scores);

        assert_eq!(scores[0], ORDER_REJECT);
        assert_eq!(scores[1], 1000);
        assert_eq!(scores[2], 1000);
        assert_eq!(scores[3], ORDER_REJECT);
        assert_eq!(scores[4], ORDER_REJECT);
    }

    #[test]
    fn test_order_mask_unwitnessed_state() {
        let dag = test_dag();
        let mut scores = [500i32; 5];

        // Current state 999 is un-witnessed in DAG.
        dag.apply_order_mask(999, &mut scores);

        for &s in scores.iter() {
            assert_eq!(s, ORDER_REJECT);
        }
    }

    #[test]
    fn test_order_dag_validate_path() {
        let dag = test_dag();

        // Valid: Flat -> OpenLong -> CloseLong.
        assert!(dag.validate_path(&[0, 1, 3]));
        // Valid: Flat -> OpenShort -> CloseShort.
        assert!(dag.validate_path(&[0, 2, 4]));

        // Invalid: Flat -> CloseLong (missing OpenLong leg).
        assert!(!dag.validate_path(&[0, 3]));
        // Invalid: OpenLong -> CloseShort (wrong close leg).
        assert!(!dag.validate_path(&[1, 4]));
        // Empty path.
        assert!(!dag.validate_path(&[]));
    }

    #[test]
    fn test_allowed_transition_max_edges_truncation() {
        let node = AllowedTransition::new(10, 0x1234, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(node.edge_count, MAX_EDGES_PER_NODE);
        assert!(node.allows_transition(8));
        assert!(!node.allows_transition(9));
    }

    #[test]
    fn test_order_dag_find_node() {
        let dag = test_dag();
        assert!(dag.find_node(0).is_some());
        assert!(dag.find_node(100).is_none());
    }
}
