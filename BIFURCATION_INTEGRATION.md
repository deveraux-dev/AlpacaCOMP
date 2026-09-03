# Bifurcation Circuit Breaker Integration Guide

## What It Does

The bifurcation detector measures approach to liquidation boundary by tracking:
- **Velocity**: How fast equity is declining
- **Acceleration**: Is the decline accelerating exponentially (backpressure building)
- **Margin to threshold**: How close to maintenance requirement
- **Backpressure**: `acceleration / available_margin` — when > 1, entropy is building faster than system can vent

## Trinary State Machine

The system operates in three states instead of binary (Safe/Unsafe):

```
+1 ACCUMULATE  → Normal trading, accept all valid orders
 0 HOLD        → Bifurcation ridge, coasting, no new orders
-1 VENT        → Active entropy release, close positions to free margin
```

This prevents catastrophic misfire. Binary "panic sell" flips to the opposite extreme and crashes the system. Trinary "HOLD" enters a neutral state that absorbs backpressure.

## Integration Steps

### 1. Feed Account Data to Governor

In your Alpaca event loop, populate health metrics:

```rust
use forge_daemon::governor::AlpacaDaemonHealth;
use std::sync::atomic::Ordering;

let health = Arc::new(AlpacaDaemonHealth::default());

// On each account update (every order execution, every minute):
let equity_cents = (account.equity * 100.0) as u64;
let maint_cents = (account.maintenance_requirement * 100.0) as u64;

health.equity_bp.store(equity_cents * 100, Ordering::Relaxed);  // basis points
health.maintenance_bp.store(maint_cents * 100, Ordering::Relaxed);
```

### 2. Spawn Governor Thread

```rust
use forge_daemon::governor::spawn_governor;

let health = Arc::new(AlpacaDaemonHealth::default());
spawn_governor(health.clone());  // Runs 1 thread, 1s tick, autonomous
```

### 3. Gate Order Dispatch to Trinary State

Modify `dispatch.rs` to check the governor state before placing orders:

```rust
// In dispatch logic
let bifurc_margin = health.bifurcation_margin.load(Ordering::Relaxed);

if bifurc_margin > 0 {
    // CIRCUIT BREAKER: liquidation risk critical
    return Err(DispatchRefusal::LiquidationBoundary);
}

// If not in crisis, check strain score for HOLD state
let total_strain = health.risk_gate_faults.load(Ordering::Relaxed) 
    + health.order_ack_misses.load(Ordering::Relaxed)
    + /* other axes */;

match total_strain {
    s if s == 0 => {
        // +1 ACCUMULATE: accept orders
        dispatch_order(...)
    }
    _ => {
        // 0 HOLD: reject new orders, wait for backpressure to dissipate
        return Err(DispatchRefusal::SystemHold);
    }
}
```

### 4. Parse Governor Stderr

The governor logs to stderr:

```
[governor] Autonomous Governor online (8 axes: 7 system + 1 bifurcation, 1s tick)
[governor] Trinary state: +1 ACCUMULATE (normal trading)
[governor] liquidation caution: margin=22.50%
[governor] BIFURCATION CIRCUIT BREAKER: backpressure=1.23 margin=14.50% accel=0.0045
[governor] Trinary State: 0 HOLD (coasting, no new orders)
```

Parse these to:
- Alert human trader
- Emit metrics (Prometheus, DataDog, etc.)
- Adjust API pacer conservatively

### 5. Recovery Protocol

When circuit breaker fires:

1. **Stop all new orders** (already enforced by gate)
2. **Monitor backpressure** — if it continues to rise, enter **-1 VENT**
3. **Micro-perturbations**: Send tiny test orders to probe market direction while in HOLD state
4. **Resume when safe**: Once `margin_to_threshold > 0.20` and `acceleration < 0.01`, return to +1 ACCUMULATE

## Tuning Parameters

In `bifurcation.rs`, adjust for your risk profile:

```rust
pub struct LiquidationDetector {
    caution_margin: f64,       // Default 0.30 (30%)
    critical_margin: f64,      // Default 0.15 (15%)
    accel_threshold: f64,      // Default 0.001 (0.1% of equity per tick)
    window_size: usize,        // Default 5 ticks for smoothing
}
```

- **Conservative**: `caution_margin = 0.40`, `critical_margin = 0.25`
- **Aggressive**: `caution_margin = 0.20`, `critical_margin = 0.10`

## Backpressure Intuition

Backpressure = `acceleration / available_margin`

- **< 0.5**: System can vent entropy comfortably, no warning
- **0.5-1.0**: Caution zone, monitor closely
- **> 1.0**: Backpressure exceeds venting capacity, entropy will accumulate, bifurcation imminent

When backpressure > 1.0 and margin is tight, the system crosses from reversible (Acute) to irreversible (Chronic) — this is the circuit breaker boundary.

## Tests

All tests pass and are production-ready:

```bash
cargo test --lib bifurcation
# test bifurcation::tests::test_safe_state ... ok
# test bifurcation::tests::test_caution_zone ... ok
# test bifurcation::tests::test_critical_and_accelerating ... ok
# test bifurcation::tests::test_liquidation_cascade ... ok
```

## Fast Governor is the Differentiator

Your existing `fast_governor` works on system health (memory, WebSocket, order latency). This bifurcation axis adds **account health** — tracking whether your capital structure itself is approaching bifurcation.

Combined, you have:
1. System survives order latency/network blips (fast governor)
2. Account survives drawdown cascades (bifurcation detector)
3. Trinary state prevents binary panic (accumulate/hold/vent)

This is the edge for the hackathon: **predictive circuit breakers before liquidation, not reactive kills after**.
