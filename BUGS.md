# arrayvec — Injected Bugs

ETNA workload for the Rust `arrayvec` crate. Each variant re-introduces
one historical bug-fix into a fresh patched branch and pairs it with a
framework-neutral property, four PBT adapters, and a deterministic
witness test.

Total mutations: 2

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `extend_silent_truncate_a554ea2_1` | `extend_silent_truncate` | `src/arrayvec.rs:1143` | `marauders` | `a554ea219a181f74542c5aaaeccec97a52f51939` |
| 2 | `insert_bound_off_by_one_2a1378d_1` | `insert_bound_off_by_one` | `src/arrayvec.rs:310` | `marauders` | `2a1378d3eb073026468063fd354d90511b61def0` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `extend_silent_truncate_a554ea2_1` | `ExtendPanicsOnOverflow` | `witness_extend_panics_on_overflow_case_five_items_cap_four`, `witness_extend_panics_on_overflow_case_exact_fit` |
| `insert_bound_off_by_one_2a1378d_1` | `InsertAtLengthSucceeds` | `witness_insert_at_length_succeeds_case_empty`, `witness_insert_at_length_succeeds_case_three_elements` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `ExtendPanicsOnOverflow` | ✓ | ✓ | ✓ | ✓ |
| `InsertAtLengthSucceeds` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. extend_silent_truncate

- **Variant**: `extend_silent_truncate_a554ea2_1`
- **Location**: `src/arrayvec.rs:1143` (inside `extend_from_iter`)
- **Property**: `ExtendPanicsOnOverflow`
- **Witness(es)**:
  - `witness_extend_panics_on_overflow_case_five_items_cap_four`
  - `witness_extend_panics_on_overflow_case_exact_fit`
- **Source**: API: Panic in .extend() and from_iter on capacity exceeded
  > `.extend()` and `from_iter` silently truncated the iterator once the `ArrayVec` hit capacity, contradicting the documented behaviour of panicking on overflow. The fix routes both paths through `extend_panic()` when the backing buffer is full.
- **Fix commit**: `a554ea219a181f74542c5aaaeccec97a52f51939` — API: Panic in .extend() and from_iter on capacity exceeded
- **Invariant violated**: `ArrayVec::extend` must panic when the iterator yields more items than the vector's remaining capacity; truncating silently is a data-loss bug visible to callers that rely on the documented "panic on overflow" behaviour.
- **How the mutation triggers**: replacing `if ptr == end_ptr && CHECK { extend_panic(); }` with `if ptr == end_ptr && CHECK { return; }` makes the extend loop bail out silently once the backing buffer is full. Witness `case_five_items_cap_four` drives this by extending a capacity-4 `ArrayVec` with a 5-element iterator: base build panics as expected; the buggy build truncates to 4 elements and returns normally, and the property reports `vec.len()=4` without a panic observation. The `case_exact_fit` witness guards against over-correction — 4 items must still succeed.

### 2. insert_bound_off_by_one

- **Variant**: `insert_bound_off_by_one_2a1378d_1`
- **Location**: `src/arrayvec.rs:310` (inside `try_insert`)
- **Property**: `InsertAtLengthSucceeds`
- **Witness(es)**:
  - `witness_insert_at_length_succeeds_case_empty`
  - `witness_insert_at_length_succeeds_case_three_elements`
- **Source**: Fix bounds checking in ArrayVec::insert(index, element)
  > `try_insert` rejected `index == len` (the valid append position) with an out-of-range panic instead of pushing to the tail, diverging from `Vec::insert`'s contract. The fix loosens the guard from `index >= len` to `index > len` while adding a separate capacity check.
- **Fix commit**: `2a1378d3eb073026468063fd354d90511b61def0` — Fix bounds checking in ArrayVec::insert(index, element)
- **Invariant violated**: `ArrayVec::try_insert(len, v)` must behave as a push-to-tail — index `== len` is a valid append position per the docs, matching `Vec::insert`'s contract. The buggy boundary check rejects this call with a panic as if the index were out of range.
- **How the mutation triggers**: the base code tests `if index > self.len() { panic_oob!(...) }`; the mutation restores the pre-2a1378d condition `if index >= self.len() { panic_oob!(...) }`, which treats the append position as out-of-bounds. `witness_case_empty` (insert at 0 into an empty vec) and `witness_case_three_elements` (insert at 3 into a 3-element vec) both hit the boundary and panic under the buggy code where the base succeeds.

## Dropped Candidates

- `27401fbc` (Fix miri error in extend_zst) — detection requires miri
- `c094906a` (Fix mutable reborrow in retain) — detection requires miri
- `c9ff5794` (Fix stacked borrows violations) — detection requires miri
- `9beb7534` (fix aliasing in drain_range) — detection requires miri
- `f40e708f` (truncate: no raw pointer from self) — detection requires miri
- `2a339799` (.extend() for ZST) — detection requires miri (ZST len)
- `9f578790` (UB in DerefMut of ArrayString) — detection requires miri
- `dc33f756` (track_caller) — observability-only, no behavior change
- `812c83a2` (16-bit lengths on 16-bit targets) — not reachable on 64-bit host
- `0fdf39f1` (debug-assert into_inner_unchecked) — unsafe API; no public invariant change
- `b7f3aa9f` (new_const) — API addition, not a bug
- `b82a6d49` (retain optimization const-generic switch) — no behavior change
- `090a5c50` (encode_utf8 uninit pointer) — surface removed, const-generic rewrite
