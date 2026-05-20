//! End-to-end demo of `crabcheck::quickcheck_with_locate!` on a real
//! faultloc benchmark variant. Activate the bug via:
//!   marauders set --variant insert_bound_off_by_one_2a1378d_1
//! then run:
//!   RUSTFLAGS="-C instrument-coverage -C link-dead-code -C codegen-units=1 -C debuginfo=2" \
//!     CRABCHECK_PROFILING_MUTATIONS=100 \
//!     CRABCHECK_PROFILING_INITIAL_PASSES=10 \
//!     CRABCHECK_PROFILING_RANDOM_ITERS=50 \
//!     cargo test --release --test locate_insert --features etna -- --nocapture
//!
//! Note: integration tests are compiled as their own crate (named after
//! this file's stem — `locate_insert`), so we use the **2-arg macro form**
//! to tell crabcheck to filter on the `arrayvec` library, not the test crate.

#![cfg(feature = "etna")]

use arrayvec::etna::{property_insert_at_length_succeeds, PropertyResult};
use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand_etna::Rng;

#[derive(Clone, Debug)]
struct InsertInput {
    existing: Vec<u32>,
    value: u32,
}

impl<R: Rng> Arbitrary<R> for InsertInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = (rng.random::<u8>() % 8) as usize;
        InsertInput {
            existing: (0..len).map(|_| rng.random::<u32>()).collect(),
            value: rng.random::<u32>(),
        }
    }
}

impl<R: Rng> Mutate<R> for InsertInput {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.clone();
        match rng.random_range(0u8..4) {
            0 => {
                let b = rng.random_range(0u32..32);
                out.value ^= 1u32 << b;
            }
            1 if !out.existing.is_empty() => {
                let i = rng.random_range(0..out.existing.len());
                let b = rng.random_range(0u32..32);
                out.existing[i] ^= 1u32 << b;
            }
            2 if out.existing.len() < 12 => out.existing.push(rng.random::<u32>()),
            _ if !out.existing.is_empty() => {
                out.existing.pop();
            }
            _ => {}
        }
        out
    }
}

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn property(i: InsertInput) -> Option<bool> {
    to_opt(property_insert_at_length_succeeds(i.existing, i.value))
}

#[test]
fn locate_insert_bound_off_by_one() {
    // 2-arg form: tell crabcheck to filter on the `arrayvec` crate, not the
    // integration-test crate `locate_insert`.
    let report = crabcheck::quickcheck_with_locate!(property, "arrayvec");

    eprintln!("{report}");
    eprintln!("\n--- raw top-3 ---");
    for s in report.suspects.iter().take(3) {
        eprintln!(
            "  rank={} file={} lines={}-{} fn={} ochiai={:.4} delta={:.2} confidence={} ({})",
            s.rank,
            s.region.file,
            s.region.start_line,
            s.region.end_line,
            s.region.function,
            s.region.suspiciousness.ochiai,
            s.region.delta,
            s.confidence,
            s.confidence_rule,
        );
    }
    eprintln!("--- diagnostics ({}) ---", report.diagnostics.len());
    for d in &report.diagnostics {
        eprintln!("  - {}", d.tag());
    }
    eprintln!("workdir: {}", report.workdir.display());

    report.assert_failed();
    let top = report.top().expect("expected at least one suspect");

    // The marauders bug is in arrayvec::ArrayVec::try_insert (around the
    // boundary check at line ~310-322 of src/arrayvec.rs). Either the
    // top suspect is in src/arrayvec.rs or it ends up at src/arrayvec/...
    assert!(
        top.region.file.ends_with("src/arrayvec.rs"),
        "expected top suspect in src/arrayvec.rs; got {}",
        top.region.file
    );
}
