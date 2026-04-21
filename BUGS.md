# arrayvec — Injected Bugs

Total mutations: 2

## Bug Index

| # | Name | Variant | File | Injection | Fix Commit |
|---|------|---------|------|-----------|------------|
| 1 | `extend_silent_truncate` | `extend_silent_truncate_a554ea2_1` | `src/arrayvec.rs:1143` | `marauders` | `a554ea219a181f74542c5aaaeccec97a52f51939` |
| 2 | `insert_bound_off_by_one` | `insert_bound_off_by_one_2a1378d_1` | `src/arrayvec.rs:310` | `marauders` | `2a1378d3eb073026468063fd354d90511b61def0` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `extend_silent_truncate_a554ea2_1` | `property_extend_panics_on_overflow` | `witness_extend_panics_on_overflow_case_five_items_cap_four`, `witness_extend_panics_on_overflow_case_exact_fit` |
| `insert_bound_off_by_one_2a1378d_1` | `property_insert_at_length_succeeds` | `witness_insert_at_length_succeeds_case_empty`, `witness_insert_at_length_succeeds_case_three_elements` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `property_extend_panics_on_overflow`     | ✓ | ✓ | ✓ | ✓ |
| `property_insert_at_length_succeeds`     | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. extend_silent_truncate

- **Variant**: `extend_silent_truncate_a554ea2_1`
- **Location**: `src/arrayvec.rs:1143` (inside `extend_from_iter`)
- **Property**: `property_extend_panics_on_overflow`
- **Witness(es)**: `witness_extend_panics_on_overflow_case_five_items_cap_four`, `witness_extend_panics_on_overflow_case_exact_fit`
- **Fix commit**: `a554ea219a181f74542c5aaaeccec97a52f51939` — `.extend() should panic when out of capacity`
- **Invariant violated**: `ArrayVec::extend` must panic when the iterator yields more items than the vector's remaining capacity; truncating silently is a data-loss bug visible to callers that rely on the documented "panic on overflow" behaviour.
- **How the mutation triggers**: replacing `if ptr == end_ptr && CHECK { extend_panic(); }` with `if ptr == end_ptr && CHECK { return; }` makes the extend loop bail out silently once the backing buffer is full. Witness `case_five_items_cap_four` drives this by extending a capacity-4 `ArrayVec` with a 5-element iterator: base build panics as expected; the buggy build truncates to 4 elements and returns normally, and the property reports `vec.len()=4` without a panic observation. The `case_exact_fit` witness guards against over-correction — 4 items must still succeed.

### 2. insert_bound_off_by_one

- **Variant**: `insert_bound_off_by_one_2a1378d_1`
- **Location**: `src/arrayvec.rs:310` (inside `try_insert`)
- **Property**: `property_insert_at_length_succeeds`
- **Witness(es)**: `witness_insert_at_length_succeeds_case_empty`, `witness_insert_at_length_succeeds_case_three_elements`
- **Fix commit**: `2a1378d3eb073026468063fd354d90511b61def0` — `Allow insertion at index == length (append)`
- **Invariant violated**: `ArrayVec::try_insert(len, v)` must behave as a push-to-tail — index `== len` is a valid append position per the docs, matching `Vec::insert`'s contract. The buggy boundary check rejects this call with a panic as if the index were out of range.
- **How the mutation triggers**: the base code tests `if index > self.len() { panic_oob!(...) }`; the mutation restores the pre-2a1378d condition `if index >= self.len() { panic_oob!(...) }`, which treats the append position as out-of-bounds. `witness_case_empty` (insert at 0 into an empty vec) and `witness_case_three_elements` (insert at 3 into a 3-element vec) both hit the boundary and panic under the buggy code where the base succeeds.
