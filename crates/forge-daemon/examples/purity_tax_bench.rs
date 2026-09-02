//! purity_tax_bench — measured receipt: N×IPR integer dot product vs Shannon
//! entropy (f64 ln) over the same 243-entry permyriad book distribution.
//! Feeds the write-up's "transcendental tax" claim with a local number.

use std::hint::black_box;
use std::time::Instant;

const N: usize = 243;
const ITERS: u32 = 1_000_000;

fn book() -> [u32; N] {
    let mut p = [0u32; N];
    let mut seed = 0x13F0u32;
    let mut total = 0u32;
    for slot in p.iter_mut() {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        *slot = (seed >> 24) + 1;
        total += *slot;
    }
    let mut acc = 0u32;
    for slot in p.iter_mut() {
        *slot = *slot * 10_000 / total;
        acc += *slot;
    }
    p[0] += 10_000 - acc;
    p
}

fn nipr_permyriad(p: &[u32; N]) -> u64 {
    let mut sum_sq = 0u64;
    for &x in p {
        sum_sq += (x as u64) * (x as u64);
    }
    (N as u64) * sum_sq / 10_000
}

fn shannon_nats(p: &[u32; N]) -> f64 {
    let mut h = 0.0f64;
    for &x in p {
        if x > 0 {
            let pf = x as f64 / 10_000.0;
            h -= pf * pf.ln();
        }
    }
    h
}

fn main() {
    let p = book();

    let t0 = Instant::now();
    let mut acc_i = 0u64;
    for _ in 0..ITERS {
        acc_i = acc_i.wrapping_add(nipr_permyriad(black_box(&p)));
    }
    let nipr_ns = t0.elapsed().as_nanos() as f64 / ITERS as f64;

    let t1 = Instant::now();
    let mut acc_f = 0.0f64;
    for _ in 0..ITERS {
        acc_f += shannon_nats(black_box(&p));
    }
    let shannon_ns = t1.elapsed().as_nanos() as f64 / ITERS as f64;

    println!("entries={N} iters={ITERS}");
    println!("nipr    : {:8.1} ns/op  (checksum {})", nipr_ns, acc_i);
    println!("shannon : {:8.1} ns/op  (checksum {:.3})", shannon_ns, acc_f);
    println!("tax     : {:8.2}x", shannon_ns / nipr_ns);
}
