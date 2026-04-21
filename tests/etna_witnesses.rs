//! Deterministic witness tests for arrayvec ETNA variants.
//!
//! Each `witness_<name>_case_<tag>` passes on the base HEAD and fails under
//! the corresponding `etna/<variant>` branch (or with `M_<variant>=active`
//! via marauders). Witnesses call `property_<name>` directly with frozen
//! inputs — no proptest/quickcheck/RNG/clock machinery.

#![cfg(feature = "etna")]

use arrayvec::etna::{
    property_extend_panics_on_overflow, property_insert_at_length_succeeds, PropertyResult,
};

fn expect_pass(r: PropertyResult, what: &str) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Fail(m) => panic!("{}: property failed: {}", what, m),
        PropertyResult::Discard => panic!("{}: unexpected discard", what),
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Variant: extend_silent_truncate_a554ea2_1
//
// `.extend(iter)` with more items than capacity must panic. The buggy
// version silently truncates. Property returns Pass iff (items.len() > cap
// ⇒ panic) ∧ (items.len() ≤ cap ⇒ exact contents).
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn witness_extend_panics_on_overflow_case_five_items_cap_four() {
    // CAP=4 inside the property; 5 items force overflow.
    expect_pass(
        property_extend_panics_on_overflow(vec![1u32, 2, 3, 4, 5]),
        "extend 5 into cap 4",
    );
}

#[test]
fn witness_extend_panics_on_overflow_case_exact_fit() {
    // 4 items fit exactly — must succeed without panic and preserve contents.
    expect_pass(
        property_extend_panics_on_overflow(vec![10u32, 20, 30, 40]),
        "extend 4 into cap 4",
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Variant: insert_bound_off_by_one_2a1378d_1
//
// `try_insert(len, value)` must succeed (index == len is a valid append
// position per docs). The buggy off-by-one check (`index >= self.len()`)
// panics on index == len.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn witness_insert_at_length_succeeds_case_empty() {
    // existing is empty, so len=0. try_insert(0, v) must behave as push.
    expect_pass(
        property_insert_at_length_succeeds(vec![], 42u32),
        "insert at index 0 on empty vec",
    );
}

#[test]
fn witness_insert_at_length_succeeds_case_three_elements() {
    // existing has 3 elements; try_insert(3, v) must behave as push-to-tail.
    expect_pass(
        property_insert_at_length_succeeds(vec![7u32, 8, 9], 99u32),
        "insert at index 3 on 3-element vec",
    );
}
