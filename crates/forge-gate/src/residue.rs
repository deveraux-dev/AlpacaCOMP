//! The R in D=T+F+R: Fredholm second-kind resolvent `(I − λK) f = g` in
//! permyriad fixed point. Ported from F:\v3\forge-core-v3\src\resolvent.rs
//! (Field5D + macaulay_pow); PMY re-homed to market_purity::PERMYRIAD_SCALE.

use crate::market_purity::PERMYRIAD_SCALE;

const PMY: u64 = PERMYRIAD_SCALE as u64;

/// Truncating `a·b / PMY` in `i128`, deterministic on every target.
#[inline(always)]
const fn scale(a: i64, b: i64) -> i128 {
    (a as i128 * b as i128) / PMY as i128
}

/// Discrete Macaulay bracket `⟨x − a⟩ⁿ` for `n ≥ 0`: saturating, integer-only
/// step/ramp/curve driver for the resolvent's input `g`. `n < 0`
/// (distributional impulses) is out of scope by design — model an impulse as
/// an explicit boundary-tick entry, never a bracket evaluation.
#[inline(always)]
pub const fn macaulay_pow(x: i64, a: i64, n: u32) -> i64 {
    let diff = x.saturating_sub(a);
    if diff > 0 { diff.saturating_pow(n) } else { 0 }
}

/// An N-channel second-kind coupling `M = λK`, permyriad entries. Constructed
/// only when the Neumann series converges: infinity norm strictly below PMY.
/// The strict `<` is load-bearing: `‖M‖∞ == 1` (conservative/undamped
/// dynamics) has no equilibrium and must be refused, never iterated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field5D<const N: usize> {
    /// Row-major coupling. `m[i][j]` = parts-per-myriad of channel `j` folded into `i`.
    m: [[i64; N]; N],
}

impl<const N: usize> Field5D<N> {
    /// Build a field, or refuse a divergent coupling. `None` when any row's
    /// absolute entries sum to PMY or more.
    pub fn new(m: [[i64; N]; N]) -> Option<Self> {
        let mut i = 0;
        while i < N {
            let mut row_abs: i128 = 0;
            let mut j = 0;
            while j < N {
                row_abs += (m[i][j] as i128).abs();
                j += 1;
            }
            if row_abs >= PMY as i128 {
                return None;
            }
            i += 1;
        }
        Some(Self { m })
    }

    /// The INVERSE operator `g = (I − M) f`. One exact O(N²) pass.
    pub fn deproject(&self, f: &[i64; N]) -> [i64; N] {
        let mut g = [0i64; N];
        for i in 0..N {
            let mut coupled: i128 = 0;
            for j in 0..N {
                coupled += scale(self.m[i][j], f[j]);
            }
            g[i] = f[i] - coupled as i64;
        }
        g
    }

    /// The resolvent `f = (I − M)⁻¹ g` by Neumann iteration `f ← g + M f` to
    /// a fixed point. `None` if unsettled within `max_iters` — a
    /// non-converging residue is a defect, never a silent best-effort.
    pub fn resolve(&self, g: &[i64; N], max_iters: u32) -> Option<[i64; N]> {
        let mut f = *g;
        for _ in 0..max_iters {
            let mut next = [0i64; N];
            for i in 0..N {
                let mut coupled: i128 = 0;
                for j in 0..N {
                    coupled += scale(self.m[i][j], f[j]);
                }
                next[i] = g[i] + coupled as i64;
            }
            if next == f {
                return Some(next);
            }
            f = next;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_zero_is_the_heaviside_step() {
        assert_eq!(macaulay_pow(4, 5, 0), 0);
        assert_eq!(macaulay_pow(5, 5, 0), 0, "0 at the boundary, not 1/2");
        assert_eq!(macaulay_pow(6, 5, 0), 1);
    }

    #[test]
    fn n_one_is_a_ramp_from_the_boundary() {
        assert_eq!(macaulay_pow(3, 5, 1), 0);
        assert_eq!(macaulay_pow(5, 5, 1), 0);
        assert_eq!(macaulay_pow(6, 5, 1), 1);
        assert_eq!(macaulay_pow(9, 5, 1), 4);
    }

    #[test]
    fn the_243_worked_example_matches_trit_capacity() {
        assert_eq!(macaulay_pow(4, 1, 5), 243);
        assert_eq!(macaulay_pow(4, 1, 5), 3i64.pow(5));
    }

    #[test]
    fn overflow_saturates_instead_of_panicking() {
        assert_eq!(macaulay_pow(i64::MAX, i64::MIN, 2), i64::MAX);
        assert_eq!(macaulay_pow(1_000_000, 0, 10), i64::MAX);
    }

    #[test]
    fn a_divergent_coupling_is_refused() {
        assert!(Field5D::new([[6_000i64, 4_000], [0, 0]]).is_none());
        assert!(Field5D::new([[6_000i64, 3_999], [0, 0]]).is_some());
        assert!(Field5D::new([[-6_000i64, -4_000], [0, 0]]).is_none());
        assert!(Field5D::new([[-5_000i64, 4_999], [0, 0]]).is_some());
    }

    #[test]
    fn the_conservative_boundary_is_refused_on_purpose() {
        // Lossless swap and undamped oscillator: ‖M‖∞ = PMY exactly, no
        // equilibrium exists. If these ever construct, the guard was wrongly
        // loosened to `<=` — fix the guard, never the test.
        assert!(Field5D::new([[0i64, 10_000], [10_000, 0]]).is_none());
        assert!(Field5D::new([[0i64, 10_000], [-10_000, 0]]).is_none());
        assert!(Field5D::new([[0i64, 9_999], [-9_999, 0]]).is_some());
    }

    #[test]
    fn the_zero_field_is_the_identity_both_ways() {
        let f = Field5D::new([[0i64; 4]; 4]).unwrap();
        let v = [7, -3, 100, 0];
        assert_eq!(f.deproject(&v), v);
        assert_eq!(f.resolve(&v, 8).unwrap(), v);
    }

    #[test]
    fn deproject_is_the_exact_operator() {
        let field = Field5D::new([[2_000i64, 1_000], [500, 3_000]]).unwrap();
        let f = [10_000i64, 20_000];
        assert_eq!(field.deproject(&f), [6_000, 13_500]);
    }

    #[test]
    fn deproject_inverts_resolve_within_flooring_distance() {
        let field = Field5D::new([[3_000i64, -1_500, 500], [800, 2_000, 1_200], [-400, 600, 2_500]])
            .unwrap();
        for g in [[10_000i64, 0, 0], [1, 2, 3], [50_000, -20_000, 30_000], [-1, -1, -1]] {
            let f = field.resolve(&g, 200).expect("converges");
            let g_back = field.deproject(&f);
            for k in 0..3 {
                assert!((g_back[k] - g[k]).abs() <= 3, "channel {k}: {} vs {}", g_back[k], g[k]);
            }
        }
    }

    #[test]
    fn resolve_inverts_deproject_within_flooring_distance() {
        let field = Field5D::new([[2_500i64, 1_000], [-900, 2_000]]).unwrap();
        for f in [[10_000i64, 5_000], [0, 100_000], [-7_000, 3_000]] {
            let g = field.deproject(&f);
            let f_back = field.resolve(&g, 200).expect("converges");
            for k in 0..2 {
                assert!((f_back[k] - f[k]).abs() <= 3, "channel {k}: {} vs {}", f_back[k], f[k]);
            }
        }
    }

    #[test]
    fn diagonal_field_is_a_leaky_integrator() {
        // M[i][i] = keep = PMY - leak; a constant drive g settles at the
        // scalar equilibrium g·PMY/leak (the source repo's LeakyPermyriad
        // identity, computed inline here — forge-gate has no decay module).
        let leaks = [100u64, 250, 500, 2_000];
        let mut m = [[0i64; 4]; 4];
        for i in 0..4 {
            m[i][i] = (PMY - leaks[i]) as i64;
        }
        let field = Field5D::new(m).unwrap();
        let g = [500i64, 500, 500, 500];
        let f = field.resolve(&g, 5_000).unwrap();
        for i in 0..4 {
            let want = 500u64 * PMY / leaks[i];
            assert!(f[i] as u64 <= want, "ch {i}: {} > {want}", f[i]);
            assert!(want - f[i] as u64 <= PMY / leaks[i] + 1, "ch {i} too far below");
        }
    }

    #[test]
    fn an_unsettled_field_refuses_rather_than_guesses() {
        let field = Field5D::new([[9_990i64]]).unwrap();
        assert!(field.resolve(&[10_000], 3).is_none(), "3 iters cannot settle a 0.1% leak");
        assert!(field.resolve(&[10_000], 100_000).is_some(), "given room, it settles");
    }

    #[test]
    fn the_field_is_deterministic() {
        let field = Field5D::new([[3_000i64, 900, -400], [1_100, 2_200, 700], [200, -600, 2_800]])
            .unwrap();
        let g = [12_345i64, -6_789, 4_242];
        assert_eq!(field.resolve(&g, 500), field.resolve(&g, 500));
        assert_eq!(field.deproject(&g), field.deproject(&g));
    }
}
