# arrayvec — ETNA Tasks

Total tasks: 8

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task.

## Task Index

| Task | Variant | Framework | Property | Witness | Command |
|------|---------|-----------|----------|---------|---------|
| 001  | `extend_silent_truncate_a554ea2_1`   | proptest   | `property_extend_panics_on_overflow`  | `witness_extend_panics_on_overflow_case_five_items_cap_four` | `cargo run --release --features etna --bin etna -- proptest ExtendPanicsOnOverflow` |
| 002  | `extend_silent_truncate_a554ea2_1`   | quickcheck | `property_extend_panics_on_overflow`  | `witness_extend_panics_on_overflow_case_five_items_cap_four` | `cargo run --release --features etna --bin etna -- quickcheck ExtendPanicsOnOverflow` |
| 003  | `extend_silent_truncate_a554ea2_1`   | crabcheck  | `property_extend_panics_on_overflow`  | `witness_extend_panics_on_overflow_case_five_items_cap_four` | `cargo run --release --features etna --bin etna -- crabcheck ExtendPanicsOnOverflow` |
| 004  | `extend_silent_truncate_a554ea2_1`   | hegel      | `property_extend_panics_on_overflow`  | `witness_extend_panics_on_overflow_case_five_items_cap_four` | `cargo run --release --features etna --bin etna -- hegel ExtendPanicsOnOverflow` |
| 005  | `insert_bound_off_by_one_2a1378d_1`  | proptest   | `property_insert_at_length_succeeds`  | `witness_insert_at_length_succeeds_case_three_elements`       | `cargo run --release --features etna --bin etna -- proptest InsertAtLengthSucceeds` |
| 006  | `insert_bound_off_by_one_2a1378d_1`  | quickcheck | `property_insert_at_length_succeeds`  | `witness_insert_at_length_succeeds_case_three_elements`       | `cargo run --release --features etna --bin etna -- quickcheck InsertAtLengthSucceeds` |
| 007  | `insert_bound_off_by_one_2a1378d_1`  | crabcheck  | `property_insert_at_length_succeeds`  | `witness_insert_at_length_succeeds_case_three_elements`       | `cargo run --release --features etna --bin etna -- crabcheck InsertAtLengthSucceeds` |
| 008  | `insert_bound_off_by_one_2a1378d_1`  | hegel      | `property_insert_at_length_succeeds`  | `witness_insert_at_length_succeeds_case_three_elements`       | `cargo run --release --features etna --bin etna -- hegel InsertAtLengthSucceeds` |

## Witness catalog

Each witness is a deterministic concrete test. Base build: passes. Variant-active build: fails.

- `witness_extend_panics_on_overflow_case_five_items_cap_four` — extend a capacity-4 `ArrayVec<u32, 4>` with `[1, 2, 3, 4, 5]`. Base panics; the variant silently truncates to 4 elements and returns.
- `witness_extend_panics_on_overflow_case_exact_fit` — extend a capacity-4 `ArrayVec<u32, 4>` with `[10, 20, 30, 40]`. Base and variant both succeed; guards against over-correcting the fix to always panic.
- `witness_insert_at_length_succeeds_case_empty` — `try_insert(0, 42)` on an empty `ArrayVec<u32, 8>`. Base returns `Ok`; the variant panics because `0 >= 0`.
- `witness_insert_at_length_succeeds_case_three_elements` — `try_insert(3, 99)` on `ArrayVec<u32, 8>` pre-loaded with `[7, 8, 9]`. Base returns `Ok` with trailing `[7, 8, 9, 99]`; the variant panics because `3 >= 3`.
