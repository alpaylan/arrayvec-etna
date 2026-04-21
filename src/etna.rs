//! ETNA framework-neutral property functions for the `arrayvec` crate.
//!
//! Each `property_<name>` is a pure function over concrete, owned inputs
//! returning `PropertyResult`. Framework adapters in `src/bin/etna.rs` and
//! witness tests in `tests/etna_witnesses.rs` call these directly — no
//! re-implementation of the invariant inside any adapter.

#![allow(missing_docs)]

use crate::ArrayVec;

use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Debug)]
pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

// ---------------------------------------------------------------------------
// extend_silent_truncate_a554ea2_1
// ---------------------------------------------------------------------------

/// Invariant: `ArrayVec<T, CAP>::extend(iter)` either panics (when `iter`
/// would exceed `CAP`) or consumes exactly the iter's items into the vec.
/// Silent truncation — dropping excess elements without a panic — is a
/// correctness bug (regression for upstream commit a554ea21 "API: Panic in
/// .extend() and from_iter on capacity exceeded").
///
/// Concretely: for `ArrayVec::<u32, CAP>::new().extend(items.iter().copied())`,
///   - if `items.len() > CAP`, the call **must** panic.
///   - if `items.len() <= CAP`, the call must not panic and the resulting vec
///     must equal `items`.
///
/// The buggy version (pre-a554ea21) silently drops items past capacity,
/// producing a short vec with no panic, and the property Fails.
pub fn property_extend_panics_on_overflow(items: Vec<u32>) -> PropertyResult {
    // Fix CAP=4 so the property is decidable for small random inputs.
    const CAP: usize = 4;

    // Cap input size so shrinking stays bounded; nothing new is learned
    // from hundreds-of-element inputs for this invariant.
    if items.len() > 32 {
        return PropertyResult::Discard;
    }

    let items_clone = items.clone();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut v: ArrayVec<u32, CAP> = ArrayVec::new();
        v.extend(items_clone.iter().copied());
        v
    }));

    if items.len() > CAP {
        // Correct behavior is a panic. No panic means silent truncation.
        match result {
            Err(_) => PropertyResult::Pass,
            Ok(v) => PropertyResult::Fail(format!(
                "extend with {} items (cap={}) did not panic; vec.len()={}",
                items.len(),
                CAP,
                v.len()
            )),
        }
    } else {
        // Fits — must not panic, and contents must match.
        match result {
            Err(_) => PropertyResult::Fail(format!(
                "extend with {} items (cap={}) panicked unexpectedly",
                items.len(),
                CAP
            )),
            Ok(v) => {
                if v.len() != items.len() {
                    return PropertyResult::Fail(format!(
                        "after extend, len={} expected {}",
                        v.len(),
                        items.len()
                    ));
                }
                for (i, (&got, want)) in v.iter().zip(items.iter()).enumerate() {
                    if got != *want {
                        return PropertyResult::Fail(format!(
                            "after extend, elem {}: got {} want {}",
                            i, got, want
                        ));
                    }
                }
                PropertyResult::Pass
            }
        }
    }
}

// ---------------------------------------------------------------------------
// insert_bound_off_by_one_2a1378d_1
// ---------------------------------------------------------------------------

/// Invariant: `ArrayVec<T, CAP>::try_insert(index, element)` accepts any
/// `index <= self.len()`; in particular `index == self.len()` is valid and
/// behaves as a push-to-tail (regression for upstream commit 2a1378d3 "Fix
/// bounds checking in ArrayVec::insert(index, element)" — the fixed check
/// must treat `index == len` as the in-bounds append case, not an OOB panic).
///
/// Concretely: build an `ArrayVec::<u32, CAP>` containing `existing`
/// (truncated/filtered to fit under capacity with at least one spare slot),
/// call `try_insert(existing.len(), value)`, and check:
///   - the call returns `Ok(())` (no panic, no CapacityError),
///   - after the call, the last element equals `value` and the prefix equals
///     `existing`.
///
/// The buggy version (`if index >= self.len() { panic }`) panics on
/// `index == len`, which `catch_unwind` captures and the property reports as
/// Fail with a counterexample.
pub fn property_insert_at_length_succeeds(existing: Vec<u32>, value: u32) -> PropertyResult {
    const CAP: usize = 8;

    if existing.len() >= CAP {
        return PropertyResult::Discard;
    }

    let existing_clone = existing.clone();
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<ArrayVec<u32, CAP>, String> {
        let mut v: ArrayVec<u32, CAP> = ArrayVec::new();
        for x in existing_clone.iter().copied() {
            if v.try_push(x).is_err() {
                return Err("unexpected capacity error during setup push".to_string());
            }
        }
        let idx = v.len();
        match v.try_insert(idx, value) {
            Ok(()) => Ok(v),
            Err(_) => Err(format!("try_insert({}, v) returned CapacityError", idx)),
        }
    }));

    match result {
        Err(_) => PropertyResult::Fail(format!(
            "try_insert(len={}, value) panicked with existing.len()={}",
            existing.len(),
            existing.len()
        )),
        Ok(Err(msg)) => PropertyResult::Fail(msg),
        Ok(Ok(v)) => {
            if v.len() != existing.len() + 1 {
                return PropertyResult::Fail(format!(
                    "after try_insert(len,v): vec.len()={} expected {}",
                    v.len(),
                    existing.len() + 1
                ));
            }
            if v[existing.len()] != value {
                return PropertyResult::Fail(format!(
                    "after try_insert(len,v): tail={} expected {}",
                    v[existing.len()],
                    value
                ));
            }
            for (i, want) in existing.iter().enumerate() {
                if v[i] != *want {
                    return PropertyResult::Fail(format!(
                        "prefix mismatch at {}: got {} want {}",
                        i, v[i], want
                    ));
                }
            }
            PropertyResult::Pass
        }
    }
}
