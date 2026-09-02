//! Autonomous Governor, retargeted from LLM-daemon process supervision to
//! Alpaca CLI subprocess supervision. Ported from
//! `F:\...\forge-daemon\src\governor.rs`. ONE thread, ONE loop, all axes
//! feed one N-dimensional `StrainScore`. Signal Law: every trip is LOUD
//! (eprintln + StrainScore increment) — silent = bug.
//!
//! Not blocked on Sean's Alpaca paper-account/API-key deferral — this is
//! pure process supervision, no live Alpaca calls. `AlpacaDaemonHealth` is a
//! scaffold: nothing populates its atomics yet, same as governor.rs's own
//! thermal axis was correctly left absent rather than faked with a stub.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

/// Shared health telemetry the (not-yet-built) Alpaca CLI daemon loop would
/// populate. Placeholder atomics — nothing writes them yet.
#[derive(Debug, Default)]
pub struct AlpacaDaemonHealth {
    /// Subprocess RSS, megabytes.
    pub rss_mb: AtomicU64,
    /// True while the Alpaca CLI subprocess is alive.
    pub alive: AtomicBool,
    /// Consecutive ticks an order ack was awaited past its deadline.
    pub order_ack_misses: AtomicU32,
    /// WebSocket reconnect attempts since the last summary window.
    pub ws_reconnects: AtomicU32,
    /// Current `api_pacer` AIMD backoff pressure, percent of ceiling.
    pub pacer_pressure_pct: AtomicU32,
    /// Count of risk_router/oracle_arbiter refusals this tick — a gate fault
    /// is strain, same Signal Law as governor.rs's sensor_faults.
    pub risk_gate_faults: AtomicU32,
}

/// N-dimensional strain score. 0 = healthy, rising = stressed.
#[derive(Debug, Default, Clone)]
pub struct StrainScore {
    pub memory: u32,
    pub order_ack_deadline: u32,
    pub ws_staleness: u32,
    pub pacer_pressure: u32,
    pub risk_gate_faults: u32,
    pub orphans_reaped: u32,
    pub sensor_faults: u32,
}

impl StrainScore {
    pub fn total(&self) -> u32 {
        self.memory + self.order_ack_deadline + self.ws_staleness + self.pacer_pressure
            + self.risk_gate_faults + self.orphans_reaped + self.sensor_faults
    }

    pub fn is_healthy(&self) -> bool {
        self.total() == 0
    }

    pub fn max_with(&mut self, other: &StrainScore) {
        self.memory = self.memory.max(other.memory);
        self.order_ack_deadline = self.order_ack_deadline.max(other.order_ack_deadline);
        self.ws_staleness = self.ws_staleness.max(other.ws_staleness);
        self.pacer_pressure = self.pacer_pressure.max(other.pacer_pressure);
        self.risk_gate_faults = self.risk_gate_faults.max(other.risk_gate_faults);
        self.orphans_reaped = self.orphans_reaped.max(other.orphans_reaped);
        self.sensor_faults = self.sensor_faults.max(other.sensor_faults);
    }
}

/// Configuration for the governor's axes.
pub struct GovernorConfig {
    pub memory_ceiling_mb: u64,
    pub order_ack_miss_threshold: u32,
    pub ws_reconnect_threshold: u32,
    pub pacer_pressure_threshold: u32,
    /// Minimum ticks between orphan-reap attempts (PID-reuse race guard).
    pub reap_cooldown_ticks: u32,
}

impl Default for GovernorConfig {
    fn default() -> Self {
        Self {
            memory_ceiling_mb: 512,
            order_ack_miss_threshold: 3,
            ws_reconnect_threshold: 5,
            pacer_pressure_threshold: 90,
            reap_cooldown_ticks: 5,
        }
    }
}

/// Spawn the governor thread. Call once when the Alpaca CLI daemon loop
/// starts.
pub fn spawn_governor(health: Arc<AlpacaDaemonHealth>) {
    std::thread::Builder::new()
        .name("governor".into())
        .spawn(move || governor_loop(health))
        .expect("governor thread");
}

fn governor_loop(health: Arc<AlpacaDaemonHealth>) {
    let cfg = GovernorConfig::default();
    let mut tick: u32 = 0;
    let mut peak = StrainScore::default();

    eprintln!("[governor] Autonomous Governor online (7 axes, 1s tick)");
    eprintln!("[governor] note: orphan-reap axis has no process-enumeration backend in this repo yet — correctly absent, not stubbed to a fake pass");

    loop {
        std::thread::sleep(Duration::from_secs(1));
        tick = tick.wrapping_add(1);

        let mut score = StrainScore::default();

        // ── Memory ──────────────────────────────────────────────────────
        let rss_mb = health.rss_mb.load(Ordering::Relaxed);
        let alive = health.alive.load(Ordering::Relaxed);
        if rss_mb == 0 && alive {
            score.sensor_faults += 1;
            eprintln!("[governor] RSS reads 0MB but subprocess is alive — sensor fault (Signal Law)");
        } else if rss_mb > cfg.memory_ceiling_mb {
            score.memory = 1;
            eprintln!("[governor] RSS {rss_mb}MB > ceiling {}MB", cfg.memory_ceiling_mb);
        }

        // ── Order-ack deadline ──────────────────────────────────────────
        let misses = health.order_ack_misses.load(Ordering::Relaxed);
        if misses >= cfg.order_ack_miss_threshold {
            score.order_ack_deadline = 1;
            eprintln!("[governor] {misses} consecutive order-ack deadline misses (threshold={})", cfg.order_ack_miss_threshold);
        }

        // ── WebSocket staleness ─────────────────────────────────────────
        let reconnects = health.ws_reconnects.load(Ordering::Relaxed);
        if reconnects >= cfg.ws_reconnect_threshold {
            score.ws_staleness = 1;
            eprintln!("[governor] {reconnects} WS reconnects this window (threshold={})", cfg.ws_reconnect_threshold);
        }

        // ── AIMD pacer pressure ─────────────────────────────────────────
        let pressure_pct = health.pacer_pressure_pct.load(Ordering::Relaxed);
        if pressure_pct >= cfg.pacer_pressure_threshold {
            score.pacer_pressure = 1;
            eprintln!("[governor] pacer pressure {pressure_pct}% (threshold={}%)", cfg.pacer_pressure_threshold);
        }

        // ── Risk-gate faults (Signal Law: a refusal is loud, not silent) ─
        let gate_faults = health.risk_gate_faults.load(Ordering::Relaxed);
        if gate_faults > 0 {
            score.risk_gate_faults = gate_faults;
            eprintln!("[governor] {gate_faults} risk-gate refusal(s) this tick");
        }

        // ── Orphan reap: correctly absent, no manufactured pass ─────────
        // No process-enumeration backend exists in this repo yet (the
        // Windows `warden` crate this axis depended on in the source
        // governor was never ported — out of scope until a real consumer
        // needs it). tick % cfg.reap_cooldown_ticks left as documentation
        // of the intended cadence, not exercised.
        let _ = cfg.reap_cooldown_ticks;

        peak.max_with(&score);

        if tick % 60 == 0 {
            if !peak.is_healthy() {
                eprintln!(
                    "[governor] StrainScore peak@{tick}: total={} (mem={} ack={} ws={} pacer={} gate={} reap={} sensor={})",
                    peak.total(), peak.memory, peak.order_ack_deadline, peak.ws_staleness,
                    peak.pacer_pressure, peak.risk_gate_faults, peak.orphans_reaped, peak.sensor_faults,
                );
            }
            peak = StrainScore::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strain_score_totals_all_axes() {
        let s = StrainScore { memory: 1, order_ack_deadline: 1, ws_staleness: 1, pacer_pressure: 1, risk_gate_faults: 2, orphans_reaped: 0, sensor_faults: 1 };
        assert_eq!(s.total(), 7);
        assert!(!s.is_healthy());
    }

    #[test]
    fn zeroed_strain_score_is_healthy() {
        assert!(StrainScore::default().is_healthy());
    }

    #[test]
    fn max_with_tracks_peak_across_ticks() {
        let mut peak = StrainScore::default();
        peak.max_with(&StrainScore { memory: 1, ..Default::default() });
        peak.max_with(&StrainScore { pacer_pressure: 1, ..Default::default() });
        // A transient spike (memory=1 on tick 1) must survive into peak even
        // though tick 2's instantaneous score no longer has memory=1.
        assert_eq!(peak.memory, 1);
        assert_eq!(peak.pacer_pressure, 1);
        assert_eq!(peak.total(), 2);
    }

    #[test]
    fn default_config_matches_stated_thresholds() {
        let cfg = GovernorConfig::default();
        assert_eq!(cfg.memory_ceiling_mb, 512);
        assert_eq!(cfg.order_ack_miss_threshold, 3);
        assert_eq!(cfg.pacer_pressure_threshold, 90);
    }
}
