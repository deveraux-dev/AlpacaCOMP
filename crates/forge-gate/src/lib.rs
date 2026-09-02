#![no_std]
#![deny(unsafe_code)]

pub mod api_pacer;
pub mod greeks;
pub mod market_collapse;
pub mod market_purity;
pub mod merkle_seal;
pub mod oracle_arbiter;
pub mod order_dag;
pub mod regime_router;
pub mod residue;
pub mod risk_router;
pub mod strategy;

#[cfg(test)]
mod integration {
    use crate::order_dag::{AllowedTransition, OrderStateDag, ORDER_REJECT};
    use crate::risk_router::{RiskRouter, EXPOSURE_LANES};

    /// Composed gate: a candidate order must clear the state-transition DAG
    /// *and* the risk router before it is allowed to execute.
    fn gate_order(
        dag: &OrderStateDag,
        current_state: u32,
        candidate_state: u32,
        exposure: &[i32; EXPOSURE_LANES],
        risk: &RiskRouter,
    ) -> bool {
        let mut scores = [0i32; 8];
        dag.apply_order_mask(current_state, &mut scores);
        let transition_allowed = scores
            .get(candidate_state as usize)
            .map_or(false, |&s| s != ORDER_REJECT);

        transition_allowed && risk.evaluate_state_safety(exposure, 1, false)
    }

    #[test]
    fn test_composed_gate_accepts_legal_low_risk_order() {
        let dag = OrderStateDag::from_nodes(&[
            AllowedTransition::new(0, 0xA001, &[1]), // Flat -> OpenLong
        ]);
        let risk = RiskRouter::new(6);
        let exposure = [0i32; EXPOSURE_LANES];

        assert!(gate_order(&dag, 0, 1, &exposure, &risk));
    }

    #[test]
    fn test_composed_gate_rejects_illegal_transition() {
        let dag = OrderStateDag::from_nodes(&[
            AllowedTransition::new(0, 0xA001, &[1]), // Flat -> OpenLong only
        ]);
        let risk = RiskRouter::new(6);
        let exposure = [0i32; EXPOSURE_LANES];

        // Candidate 2 (OpenShort) is not witnessed from state 0.
        assert!(!gate_order(&dag, 0, 2, &exposure, &risk));
    }

    #[test]
    fn test_composed_gate_rejects_legal_transition_over_margin() {
        let dag = OrderStateDag::from_nodes(&[
            AllowedTransition::new(0, 0xA001, &[1]),
        ]);
        let risk = RiskRouter::new(3);
        // 7 lanes at max magnitude 1: composite exposure 7 > margin limit 3.
        let exposure = [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0];

        assert!(!gate_order(&dag, 0, 1, &exposure, &risk));
    }

    #[test]
    fn test_composed_gate_rejects_order_that_clears_margin_but_trips_max_loss_veto() {
        use crate::risk_router::exceeds_max_loss_veto;

        let dag = OrderStateDag::from_nodes(&[
            AllowedTransition::new(0, 0xA001, &[1]),
        ]);
        let risk = RiskRouter::new(6);
        let exposure = [0i32; EXPOSURE_LANES];

        // Passes the state-transition DAG and the margin bound...
        assert!(gate_order(&dag, 0, 1, &exposure, &risk));
        // ...but a wide-strike, thin-credit spread still trips the final
        // max-loss veto and must never reach dispatch.
        assert!(exceeds_max_loss_veto(30.0, 1.00, 100_000.0));
    }
}
