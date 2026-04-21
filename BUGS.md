# tinyvec — Injected Bugs

Total mutations: 4

## Bug Index

| # | Name | Variant | File | Injection | Fix Commit |
|---|------|---------|------|-----------|------------|
| 1 | `debug_alternate_empty` | `debug_alternate_empty_a711c72_1` | `src/arrayvec.rs:1839` | `marauders` | `a711c72eef6d555ebc7bbbe78bf5039e72f790ac` |
| 2 | `remove_past_end_silent` | `remove_past_end_silent_fd3c92c_1` | `src/arrayvec.rs:818` | `marauders` | `fd3c92c35109a4b025738fe71bb0fd739c3d6002` |
| 3 | `swap_remove_last` | `swap_remove_last_71ad62a_1` | `src/arrayvec.rs:1168` | `marauders` | `71ad62a90f2ff95dae4e43d646a55b0329b1eedc` |
| 4 | `drain_end_off_by_one` | `drain_end_off_by_one_9117614_1` | `src/arrayvec_drain.rs:26` | `marauders` | `9117614aa9d527a122dff8828e56c17d247e3f5a` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `debug_alternate_empty_a711c72_1` | `property_arrayvec_debug_matches_slice` | `witness_arrayvec_debug_matches_slice_case_empty`, `witness_arrayvec_debug_matches_slice_case_three_elements` |
| `remove_past_end_silent_fd3c92c_1` | `property_remove_past_end_panics` | `witness_remove_past_end_panics_case_single_element`, `witness_remove_past_end_panics_case_three_elements` |
| `swap_remove_last_71ad62a_1` | `property_swap_remove_last_returns_tail` | `witness_swap_remove_last_returns_tail_case_single`, `witness_swap_remove_last_returns_tail_case_four_elements` |
| `drain_end_off_by_one_9117614_1` | `property_drain_matches_slice_range` | `witness_drain_matches_slice_range_case_exclusive_middle`, `witness_drain_matches_slice_range_case_full_range` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `property_arrayvec_debug_matches_slice`    | ✓ | ✓ | ✓ | ✓ |
| `property_remove_past_end_panics`          | ✓ | ✓ | ✓ | ✓ |
| `property_swap_remove_last_returns_tail`   | ✓ | ✓ | ✓ | ✓ |
| `property_drain_matches_slice_range`       | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. debug_alternate_empty

- **Variant**: `debug_alternate_empty_a711c72_1`
- **Location**: `src/arrayvec.rs:1839` (inside `impl<A: Array> Debug for ArrayVec<A>`)
- **Property**: `property_arrayvec_debug_matches_slice`
- **Witness(es)**: `witness_arrayvec_debug_matches_slice_case_empty`, `witness_arrayvec_debug_matches_slice_case_three_elements`
- **Fix commit**: `a711c72eef6d555ebc7bbbe78bf5039e72f790ac` — `fix Debug alternate mode for empty containers`
- **Invariant violated**: `format!("{:#?}", av)` must match `format!("{:#?}", av.as_slice())` — `ArrayVec`'s `Debug` impl must agree with the underlying slice in both plain and alternate modes, regardless of content.
- **How the mutation triggers**: the fix replaced the manual impl with `<[A::Item] as Debug>::fmt(self.as_slice(), f)`. The mutation reinstates the pre-a711c72 manual impl which unconditionally emits a leading `"\n    "` and a trailing `",\n"` in alternate mode. On an empty vec this produces `"[\n    ,\n]"` instead of the slice's `"[]"`, and on a populated vec the comma/newline layout differs from `[T]`'s default. `case_empty` exposes the stray comma; `case_three_elements` exposes the layout mismatch for non-empty input.

### 2. remove_past_end_silent

- **Variant**: `remove_past_end_silent_fd3c92c_1`
- **Location**: `src/arrayvec.rs:818` (inside `ArrayVec::remove`)
- **Property**: `property_remove_past_end_panics`
- **Witness(es)**: `witness_remove_past_end_panics_case_single_element`, `witness_remove_past_end_panics_case_three_elements`
- **Fix commit**: `fd3c92c35109a4b025738fe71bb0fd739c3d6002` — `Test and fix removal at past-the-end index`
- **Invariant violated**: `ArrayVec::remove(index)` must panic when `index >= self.len()`, matching the `Vec::remove` contract. Silently returning a default-constructed item — and decrementing `self.len` — is both data-loss and a length-invariant violation.
- **How the mutation triggers**: the pre-fd3c92c body iterates `targets[index..].iter_mut().rev()` and returns the final `spare`. When `index == self.len()`, `targets` is empty, the iteration is a no-op, and `spare` stays `A::Item::default()`; the function then decrements `self.len` and returns the default. The fix reads `targets[0]` unconditionally, which panics on the empty slice. `case_single_element` (`remove(1)` on `[42]`) and `case_three_elements` (`remove(3)` on `[1,2,3]`) both hit `index == len` exactly.

### 3. swap_remove_last

- **Variant**: `swap_remove_last_71ad62a_1`
- **Location**: `src/arrayvec.rs:1168` (inside `ArrayVec::swap_remove`)
- **Property**: `property_swap_remove_last_returns_tail`
- **Witness(es)**: `witness_swap_remove_last_returns_tail_case_single`, `witness_swap_remove_last_returns_tail_case_four_elements`
- **Fix commit**: `71ad62a90f2ff95dae4e43d646a55b0329b1eedc` — `Fix ArrayishVec::swap_remove for last element`
- **Invariant violated**: `swap_remove(len - 1)` must return the tail element without panicking, matching the `Vec::swap_remove` contract. Panicking on a valid in-range index is both a safety-contract violation and a correctness regression.
- **How the mutation triggers**: the pre-71ad62a body unconditionally runs `let i = self.pop().unwrap(); replace(&mut self[index], i)`. When `index == len - 1`, the `pop` call shrinks the vec to `len - 1`, and `self[index]` then indexes the (now out-of-range) final slot, triggering the slice-index panic. The fix short-circuits: when `index == len - 1`, just `pop().unwrap()`. `case_single` (`swap_remove(0)` on `[99]`) and `case_four_elements` (`swap_remove(3)` on `[1,2,3,4]`) both land on the tail-index boundary.

### 4. drain_end_off_by_one

- **Variant**: `drain_end_off_by_one_9117614_1`
- **Location**: `src/arrayvec_drain.rs:26` (inside `ArrayVecDrain::new`)
- **Property**: `property_drain_matches_slice_range`
- **Witness(es)**: `witness_drain_matches_slice_range_case_exclusive_middle`, `witness_drain_matches_slice_range_case_full_range`
- **Fix commit**: `9117614aa9d527a122dff8828e56c17d247e3f5a` — `Fix ArrayishVec::drain implementation (end bound off by one)`
- **Invariant violated**: `av.drain(a..b).collect::<Vec<_>>()` must equal `items[a..b].to_vec()` (and the remaining vec must equal `items[..a] ++ items[b..]`), matching `Vec::drain`'s semantics for both half-open and inclusive ranges.
- **How the mutation triggers**: the pre-9117614 end-bound mapping was swapped: `Bound::Included(x) => *x` (should be `x + 1`) and `Bound::Excluded(x) => x - 1` (should be `*x`). Half-open `a..b` then drains `a..b-1`, and inclusive `a..=b` drains `a..b` instead of `a..=b`. `case_exclusive_middle` (`drain(1..3)` on `[1..=5]`) exposes the half-open bug by missing the element at index 2; `case_full_range` (`drain(0..3)` on `[10,20,30]`) exposes the same mapping by drop-losing the final element.
