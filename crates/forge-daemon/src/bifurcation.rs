// Liquidation boundary detector: measures approach to irreversible account collapse
// Integrates with governor strain-score system as bifurcation_margin axis

use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LiquidationRisk {
    Safe,           // margin_to_threshold > 30%
    Caution,        // 15% < margin < 30%
    Critical,       // margin < 15% AND acceleration > threshold
    CircuitBreaker, // irreversible trajectory engaged
}

#[derive(Clone, Copy, Debug)]
pub struct BifurcationMetrics {
    pub account_equity: f64,
    pub maintenance_requirement: f64,
    pub margin_to_threshold: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub backpressure: f64,          // entropy accumulation rate vs vent capacity
    pub risk_state: LiquidationRisk,
}

pub struct LiquidationDetector {
    history: VecDeque<f64>,
    maintenance_requirement: f64,
    window_size: usize,
    caution_margin: f64,
    critical_margin: f64,
    accel_threshold: f64,
}

impl LiquidationDetector {
    pub fn new(maintenance_requirement: f64) -> Self {
        Self {
            history: VecDeque::with_capacity(5),
            maintenance_requirement,
            window_size: 5,
            caution_margin: 0.30,
            critical_margin: 0.15,
            accel_threshold: 0.001, // fraction of equity per tick, not raw dollars
        }
    }

    pub fn ingest(&mut self, equity: f64) -> BifurcationMetrics {
        self.history.push_back(equity);
        if self.history.len() > self.window_size {
            self.history.pop_front();
        }
        self.compute_metrics()
    }

    fn compute_metrics(&self) -> BifurcationMetrics {
        let equity = *self.history.back().unwrap_or(&0.0);
        let available_margin = (equity - self.maintenance_requirement).max(0.0);
        let margin_pct = if equity > 0.0 {
            available_margin / equity
        } else {
            0.0
        };

        let velocity = if self.history.len() >= 2 {
            let prev = self.history[self.history.len() - 2];
            (equity - prev).abs()
        } else {
            0.0
        };

        let acceleration = if self.history.len() >= 3 {
            let v2 = (self.history[self.history.len() - 1] - self.history[self.history.len() - 2]).abs();
            let v1 = (self.history[self.history.len() - 2] - self.history[self.history.len() - 3]).abs();
            (v2 - v1).abs()
        } else {
            0.0
        };

        let accel_relative = if equity > 0.0 {
            acceleration / equity
        } else {
            0.0
        };

        let backpressure = if available_margin > 0.01 {
            acceleration / available_margin
        } else {
            f64::INFINITY
        };

        let risk_state = if margin_pct < self.critical_margin && accel_relative > self.accel_threshold && backpressure > 1.0 {
            LiquidationRisk::CircuitBreaker
        } else if margin_pct < self.critical_margin {
            LiquidationRisk::Critical
        } else if margin_pct < self.caution_margin {
            LiquidationRisk::Caution
        } else {
            LiquidationRisk::Safe
        };

        BifurcationMetrics {
            account_equity: equity,
            maintenance_requirement: self.maintenance_requirement,
            margin_to_threshold: margin_pct,
            velocity,
            acceleration,
            backpressure,
            risk_state,
        }
    }

    pub fn reset(&mut self) {
        self.history.clear();
    }

    pub fn set_maintenance(&mut self, maintenance_requirement: f64) {
        self.maintenance_requirement = maintenance_requirement;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_state() {
        let mut detector = LiquidationDetector::new(10000.0);
        let m = detector.ingest(50000.0);
        assert_eq!(m.risk_state, LiquidationRisk::Safe);
        assert!(m.margin_to_threshold > 0.30);
    }

    #[test]
    fn test_caution_zone() {
        let mut detector = LiquidationDetector::new(10000.0);
        detector.ingest(13000.0);
        let m = detector.ingest(13000.0);
        assert_eq!(m.risk_state, LiquidationRisk::Caution);
        assert!(m.margin_to_threshold < 0.30 && m.margin_to_threshold > 0.15);
    }

    #[test]
    fn test_critical_and_accelerating() {
        let mut detector = LiquidationDetector::new(42000.0);
        detector.ingest(50000.0);
        detector.ingest(47000.0);
        let m = detector.ingest(42500.0);

        assert_eq!(m.risk_state, LiquidationRisk::CircuitBreaker);
        assert!(m.margin_to_threshold < 0.15);
        assert!(m.backpressure > 1.0);
    }

    #[test]
    fn test_critical_without_acceleration() {
        let mut detector = LiquidationDetector::new(43000.0);
        detector.ingest(50000.0);
        detector.ingest(50000.0);
        let m = detector.ingest(50000.0);

        assert_eq!(m.risk_state, LiquidationRisk::Critical);
        assert!(m.margin_to_threshold < 0.15);
        assert!(m.acceleration < 0.001);
    }

    #[test]
    fn test_liquidation_cascade() {
        let mut detector = LiquidationDetector::new(42000.0);
        let sequence = vec![50000.0, 47000.0, 42500.0];

        let mut final_m = None;
        for equity in sequence {
            let m = detector.ingest(equity);
            eprintln!("equity={} margin={:.2}% accel_rel={:.6} backpressure={:.2} risk={:?}",
                equity, m.margin_to_threshold * 100.0, m.acceleration / equity.max(1.0), m.backpressure, m.risk_state);
            final_m = Some(m);
        }

        let m = final_m.unwrap();
        assert_eq!(m.risk_state, LiquidationRisk::CircuitBreaker);
        assert!(m.margin_to_threshold < 0.15);
        assert!(m.backpressure > 1.0);
    }
}
