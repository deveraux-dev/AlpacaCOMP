//! Risk state validator — bounds-check and severity-trip gate.
//!
//! Ported from Nistam `forge-envelope::safety_router::SafetyRouter`: validates
//! a fixed-width exposure vector is structurally sound, then rejects if
//! composite exposure exceeds a configured margin limit.

/// Number of tracked exposure lanes (e.g. per-symbol or per-leg position size).
pub const EXPOSURE_LANES: usize = 13;

/// Structural risk router enforcing strict position/exposure boundaries.
pub struct RiskRouter {
    max_margin_limit: i32,
}

impl RiskRouter {
    /// Create a fresh risk router with a given composite margin limit.
    pub fn new(max_margin_limit: i32) -> Self {
        Self { max_margin_limit }
    }

    /// Validates that every exposure lane is bounded within `[-limit, +limit]`
    /// where `limit` is expressed as a fraction of max position size in
    /// permyriad (1/10000) units, here fixed at unit lane bound `[-1, 1]`
    /// pre-scale — callers scale lanes before passing in.
    pub fn validate_position_bounds(&self, exposure: &[i32; EXPOSURE_LANES], lane_bound: i32) -> bool {
        for &lane in exposure.iter() {
            if lane < -lane_bound || lane > lane_bound {
                return false; // Out-of-bounds exposure: structurally unsafe
            }
        }
        true
    }

    /// Evaluates structural safety of a verified exposure vector.
    ///
    /// If composite exposure exceeds the margin limit, or an escalation
    /// override is set, rejects the order state (equivalent to Nistam's
    /// 2-expert debate fallback: here, require manual risk-desk review).
    pub fn evaluate_state_safety(
        &self,
        exposure: &[i32; EXPOSURE_LANES],
        lane_bound: i32,
        force_reject: bool,
    ) -> bool {
        if !self.validate_position_bounds(exposure, lane_bound) {
            return false; // Structurally unsafe
        }

        let mut composite_exposure = 0i32;
        for &lane in exposure.iter() {
            composite_exposure += lane.abs();
        }

        if composite_exposure > self.max_margin_limit || force_reject {
            return false; // Margin trip: require manual risk-desk escalation
        }

        true // Safe
    }
}

/// RAMUSPRIME max-loss veto (spec section 6): the final fail-safe before
/// dispatch. `strike_width` is the distance between short and long strikes
/// on one side of the spread (dollars), `credit` is the net credit received
/// per contract (dollars), `account_balance` is the account's current
/// dollar balance. Rejects if max possible loss exceeds 2% of the account.
///
/// Max loss per contract = (strike_width * 100) - (credit * 100), since
/// options contracts are 100-share multiplier.
pub fn exceeds_max_loss_veto(strike_width: f64, credit: f64, account_balance: f64) -> bool {
    let max_loss = (strike_width * 100.0) - (credit * 100.0);
    max_loss > 0.02 * account_balance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_loss_veto_trips_over_two_percent() {
        // Strike width 5.0, credit 1.00: max loss = 500 - 100 = 400.
        // 2% of $100,000 = $2,000: 400 < 2000, should NOT trip.
        assert!(!exceeds_max_loss_veto(5.0, 1.00, 100_000.0));

        // Strike width 30.0, credit 1.00: max loss = 3000 - 100 = 2900.
        // 2900 > 2000, should trip.
        assert!(exceeds_max_loss_veto(30.0, 1.00, 100_000.0));
    }

    #[test]
    fn max_loss_veto_boundary_is_strictly_greater_than() {
        // Max loss exactly equal to 2% must NOT trip (spec says "> 2%").
        // Strike width 5.0, credit 0.0: max loss = 500. 2% of $25,000 = 500.
        assert!(!exceeds_max_loss_veto(5.0, 0.0, 25_000.0));
        // One cent over the limit trips it.
        assert!(exceeds_max_loss_veto(5.0, -0.01, 25_000.0));
    }

    #[test]
    fn test_risk_router_validation() {
        let router = RiskRouter::new(6);
        let valid_exposure = [0i32; EXPOSURE_LANES];
        assert!(router.validate_position_bounds(&valid_exposure, 1));

        let mut invalid_exposure = [0i32; EXPOSURE_LANES];
        invalid_exposure[5] = 2; // corrupt state: exceeds lane bound of 1
        assert!(!router.validate_position_bounds(&invalid_exposure, 1));
    }

    #[test]
    fn test_risk_router_evaluation() {
        let router = RiskRouter::new(6);
        let exposure = [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0]; // 7 lanes at max: composite 7
        assert!(!router.evaluate_state_safety(&exposure, 1, false));
    }
}
