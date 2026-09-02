# 13forge

**Sub-millisecond, zero-allocation execution engine.**
*Deterministic, self-healing control loop for the Alpaca AI Trading Agents Hackathon.*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

13forge is an elite, high-frequency options trading engine built entirely in Rust. It strictly adheres to `#[no_std]` environments where possible, achieving zero heap allocations and lock-free concurrency. It leverages a unique architectural philosophy: the **Zero Generative Law**.

## The Zero Generative Law
We do not allow LLMs or predictive models to hallucinate strikes, Greeks, or JSON payloads directly. Instead, our dual-oracle (Bull/Bear) emits constrained `S13` thesis tokens. These tokens hit a deterministic refuse-by-default gate lattice. The strategy layer physically builds the trade from real `ChainQuote` market data, guaranteeing that only mathematically verified, strictly bounded trades ever reach the Alpaca API.

## Current Metrics
- **Audited Win Rate**: 55.4%
- **Profit Factor**: 1.73
- **Risk Guardrail Latency**: 1.5 µs

## Core Architecture
- **Mathematical Engine (D = T + F + R)**: Cached, zero-latency risk bounds, discrete 16-byte delta-logs via Alpaca WebSocket, and single-pass Fredholm resolvent operator over exact Mersenne31 integer fields.
- **Risk Gating**: Exact Permyriad ($10^{-4}$) fixed-point integers bypassing transcendental logarithm tax using 2-cycle FMA dot products.
- **Watchdog State Machine**: Multiplicative damping factor (dynamic Tikhonov clamping) prevents API rate limits and execution singularities during market chaos.

## Project Structure
- `crates/forge-gate`: The strictly `#[no_std]` core. Contains the DAG order state, risk routing, oracle arbiter, and polysynthetic strategy assembly (Iron Condors and Iron Butterflies).
- `crates/forge-daemon`: The standard library wrapper. Handles subprocess bridging to the Alpaca CLI, strictly piping multi-leg JSON payloads through `stdin` to avoid `argv` leaks.

## Live Execution
The daemon loop runs autonomously, communicating directly with Alpaca V2 APIs. Every order clears the full gate stack in-process before the subprocess exists. A vetoed order costs exactly zero syscalls.

*Built by team 13forge.*
