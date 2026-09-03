//! Purity smoother: exponential-weighted smoothing of historical pmy via Field5D.
//! Lag-channel design (ch_i = pmy[t-i]); leaks increase with lag → recent readings weighted more.
//! Unwired, advisory only — pending decision on live-gate integration.

use crate::market_purity::PERMYRIAD_SCALE;
use crate::residue::Field5D;

const HISTORY_LEN: usize = 4;
const PMY_SCALE: i64 = PERMYRIAD_SCALE as i64;

/// Smooth a history of pmy readings via leaky-integrator field.
/// Input: last 4 pmy snapshots (most recent first). Output: u16 permyriad [0, 10000].
/// Field weights recent readings higher (lower leak rates → higher amplitude in resolvent).
/// Returns weighted average of field-resolved amplitudes, dampening spikes.
pub fn smooth_purity(history: &[u16; HISTORY_LEN]) -> u16 {
    let g: [i64; HISTORY_LEN] = [
        history[0] as i64,
        history[1] as i64,
        history[2] as i64,
        history[3] as i64,
    ];

    let mut m = [[0i64; HISTORY_LEN]; HISTORY_LEN];
    let leaks = [100i64, 200, 400, 800];
    for i in 0..HISTORY_LEN {
        m[i][i] = PMY_SCALE - leaks[i];
    }

    let field = Field5D::new(m).expect("field coupling valid");
    let f = field.resolve(&g, 1000).expect("resolves within limit");

    let sum_f: i64 = f.iter().sum();
    let sum_weights: i64 = leaks.iter().map(|&leak| PMY_SCALE / leak).sum();

    let smoothed = sum_f / sum_weights;
    (smoothed.max(0) as u16).min(10_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_sequence_preserves_value() {
        let hist = [5000u16; HISTORY_LEN];
        let smoothed = smooth_purity(&hist);
        assert!((smoothed as i32 - 5000i32).abs() <= 15, "constant input ≈ constant output, got {}", smoothed);
    }

    #[test]
    fn sharp_recent_spike_is_dampened() {
        let hist = [9000u16, 5000, 5000, 5000];
        let smoothed = smooth_purity(&hist);
        assert!(
            smoothed > 5000 && smoothed < 9000,
            "spike dampened to {}, in range (5000, 9000)",
            smoothed
        );
    }

    #[test]
    fn recent_change_dominates_old_spike() {
        let old_spike = [5000u16, 9000, 5000, 5000];
        let recent_spike = [9000u16, 5000, 5000, 5000];

        let smooth_old = smooth_purity(&old_spike);
        let smooth_recent = smooth_purity(&recent_spike);

        assert!(
            smooth_recent > smooth_old,
            "recent spike {} > old spike {}",
            smooth_recent,
            smooth_old
        );
    }

    #[test]
    fn sustained_trend_shifts_output() {
        let trending_up = [7000u16, 6500, 6000, 5500];
        let stable = [5000u16; HISTORY_LEN];

        let smooth_up = smooth_purity(&trending_up);
        let smooth_stable = smooth_purity(&stable);

        assert!(smooth_up > smooth_stable, "uptrend {} > stable {}", smooth_up, smooth_stable);
    }

    #[test]
    fn extreme_values_clamp_to_permyriad_range() {
        // Test that output never exceeds pmy range even with extreme inputs
        let hist = [10000u16, 10000, 10000, 10000];
        let smoothed = smooth_purity(&hist);
        assert_eq!(smoothed, 10000);

        let hist_zero = [0u16; HISTORY_LEN];
        let smoothed_zero = smooth_purity(&hist_zero);
        assert_eq!(smoothed_zero, 0);
    }
}
