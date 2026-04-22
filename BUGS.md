# tinyvec — Injected Bugs

ETNA workload for the Rust `tinyvec` crate. Each variant re-introduces
one historical bug-fix into a fresh patched branch and pairs it with a
framework-neutral property, four PBT adapters, and a deterministic
witness test.

Total mutations: 4

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `debug_alternate_empty_a711c72_1` | `debug_alternate_empty` | `src/arrayvec.rs:1839` | `marauders` | `a711c72eef6d555ebc7bbbe78bf5039e72f790ac` |
| 2 | `drain_end_off_by_one_9117614_1` | `drain_end_off_by_one` | `src/arrayvec_drain.rs:26` | `marauders` | `9117614aa9d527a122dff8828e56c17d247e3f5a` |
| 3 | `remove_past_end_silent_fd3c92c_1` | `remove_past_end_silent` | `src/arrayvec.rs:818` | `marauders` | `fd3c92c35109a4b025738fe71bb0fd739c3d6002` |
| 4 | `swap_remove_last_71ad62a_1` | `swap_remove_last` | `src/arrayvec.rs:1168` | `marauders` | `71ad62a90f2ff95dae4e43d646a55b0329b1eedc` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `debug_alternate_empty_a711c72_1` | `ArrayvecDebugMatchesSlice` | `witness_arrayvec_debug_matches_slice_case_empty`, `witness_arrayvec_debug_matches_slice_case_three_elements` |
| `drain_end_off_by_one_9117614_1` | `DrainMatchesSliceRange` | `witness_drain_matches_slice_range_case_exclusive_middle`, `witness_drain_matches_slice_range_case_full_range` |
| `remove_past_end_silent_fd3c92c_1` | `RemovePastEndPanics` | `witness_remove_past_end_panics_case_single_element`, `witness_remove_past_end_panics_case_three_elements` |
| `swap_remove_last_71ad62a_1` | `SwapRemoveLastReturnsTail` | `witness_swap_remove_last_returns_tail_case_single`, `witness_swap_remove_last_returns_tail_case_four_elements` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `ArrayvecDebugMatchesSlice` | ✓ | ✓ | ✓ | ✓ |
| `DrainMatchesSliceRange` | ✓ | ✓ | ✓ | ✓ |
| `RemovePastEndPanics` | ✓ | ✓ | ✓ | ✓ |
| `SwapRemoveLastReturnsTail` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. debug_alternate_empty

- **Variant**: `debug_alternate_empty_a711c72_1`
- **Location**: `src/arrayvec.rs:1839` (inside `impl<A: Array> Debug for ArrayVec<A>`)
- **Property**: `ArrayvecDebugMatchesSlice`
- **Witness(es)**:
  - `witness_arrayvec_debug_matches_slice_case_empty`
  - `witness_arrayvec_debug_matches_slice_case_three_elements`
- **Source**: [#162](https://github.com/Lokathor/tinyvec/pull/162) — fix `Debug` alternate mode for empty containers (#162)
  > `println!("{:#?}", tiny_vec!([u8; 16]))` was printing `[\n    ,\n]` instead of `[]`. The manual Debug impl unconditionally emitted separator/newline tokens regardless of length.
- **Fix commit**: `a711c72eef6d555ebc7bbbe78bf5039e72f790ac` — fix `Debug` alternate mode for empty containers (#162)
- **Invariant violated**: `format!("{:#?}", av)` must match `format!("{:#?}", av.as_slice())` — `ArrayVec`'s `Debug` impl must agree with the underlying slice in both plain and alternate modes, regardless of content.
- **How the mutation triggers**: the fix replaced the manual impl with `<[A::Item] as Debug>::fmt(self.as_slice(), f)`. The mutation reinstates the pre-a711c72 manual impl which unconditionally emits a leading `"\n    "` and a trailing `",\n"` in alternate mode. On an empty vec this produces `"[\n    ,\n]"` instead of the slice's `"[]"`, and on a populated vec the comma/newline layout differs from `[T]`'s default. `case_empty` exposes the stray comma; `case_three_elements` exposes the layout mismatch for non-empty input.

### 2. drain_end_off_by_one

- **Variant**: `drain_end_off_by_one_9117614_1`
- **Location**: `src/arrayvec_drain.rs:26` (inside `ArrayVecDrain::new`)
- **Property**: `DrainMatchesSliceRange`
- **Witness(es)**:
  - `witness_drain_matches_slice_range_case_exclusive_middle`
  - `witness_drain_matches_slice_range_case_full_range`
- **Source**: [#14](https://github.com/Lokathor/tinyvec/pull/14) — Fix ArrayishVec::drain implementation
  > Model-based fuzzing against `Vec` found an underflow while calculating the end index for `drain`; the `Bound::Included`/`Bound::Excluded` mappings were swapped, so half-open `a..b` drained `a..b-1` and inclusive `a..=b` drained `a..b`.
- **Fix commit**: `9117614aa9d527a122dff8828e56c17d247e3f5a` — Fix ArrayishVec::drain implementation
- **Invariant violated**: `av.drain(a..b).collect::<Vec<_>>()` must equal `items[a..b].to_vec()` (and the remaining vec must equal `items[..a] ++ items[b..]`), matching `Vec::drain`'s semantics for both half-open and inclusive ranges.
- **How the mutation triggers**: the pre-9117614 end-bound mapping was swapped: `Bound::Included(x) => *x` (should be `x + 1`) and `Bound::Excluded(x) => x - 1` (should be `*x`). Half-open `a..b` then drains `a..b-1`, and inclusive `a..=b` drains `a..b` instead of `a..=b`. `case_exclusive_middle` (`drain(1..3)` on `[1..=5]`) exposes the half-open bug by missing the element at index 2; `case_full_range` (`drain(0..3)` on `[10,20,30]`) exposes the same mapping by drop-losing the final element.

### 3. remove_past_end_silent

- **Variant**: `remove_past_end_silent_fd3c92c_1`
- **Location**: `src/arrayvec.rs:818` (inside `ArrayVec::remove`)
- **Property**: `RemovePastEndPanics`
- **Witness(es)**:
  - `witness_remove_past_end_panics_case_single_element`
  - `witness_remove_past_end_panics_case_three_elements`
- **Source**: [#29](https://github.com/Lokathor/tinyvec/pull/29), [#28](https://github.com/Lokathor/tinyvec/issues/28) — Test and fix removal at past-the-end index
  > `ArrayVec::remove(len)` silently decremented `len` and returned a default-constructed item instead of panicking, diverging from `Vec::remove`'s contract.
- **Fix commit**: `fd3c92c35109a4b025738fe71bb0fd739c3d6002` — Test and fix removal at past-the-end index
- **Invariant violated**: `ArrayVec::remove(index)` must panic when `index >= self.len()`, matching the `Vec::remove` contract. Silently returning a default-constructed item — and decrementing `self.len` — is both data-loss and a length-invariant violation.
- **How the mutation triggers**: the pre-fd3c92c body iterates `targets[index..].iter_mut().rev()` and returns the final `spare`. When `index == self.len()`, `targets` is empty, the iteration is a no-op, and `spare` stays `A::Item::default()`; the function then decrements `self.len` and returns the default. The fix reads `targets[0]` unconditionally, which panics on the empty slice. `case_single_element` (`remove(1)` on `[42]`) and `case_three_elements` (`remove(3)` on `[1,2,3]`) both hit `index == len` exactly.

### 4. swap_remove_last

- **Variant**: `swap_remove_last_71ad62a_1`
- **Location**: `src/arrayvec.rs:1168` (inside `ArrayVec::swap_remove`)
- **Property**: `SwapRemoveLastReturnsTail`
- **Witness(es)**:
  - `witness_swap_remove_last_returns_tail_case_single`
  - `witness_swap_remove_last_returns_tail_case_four_elements`
- **Source**: [#15](https://github.com/Lokathor/tinyvec/pull/15) — Fix ArrayishVec::swap_remove for last element
  > `swap_remove(len - 1)` panicked because the implementation popped first (shrinking the vec) then indexed `self[index]`, which is now out of range. Fixed by short-circuiting on the tail-index case.
- **Fix commit**: `71ad62a90f2ff95dae4e43d646a55b0329b1eedc` — Fix ArrayishVec::swap_remove for last element
- **Invariant violated**: `swap_remove(len - 1)` must return the tail element without panicking, matching the `Vec::swap_remove` contract. Panicking on a valid in-range index is both a safety-contract violation and a correctness regression.
- **How the mutation triggers**: the pre-71ad62a body unconditionally runs `let i = self.pop().unwrap(); replace(&mut self[index], i)`. When `index == len - 1`, the `pop` call shrinks the vec to `len - 1`, and `self[index]` then indexes the (now out-of-range) final slot, triggering the slice-index panic. The fix short-circuits: when `index == len - 1`, just `pop().unwrap()`. `case_single` (`swap_remove(0)` on `[99]`) and `case_four_elements` (`swap_remove(3)` on `[1,2,3,4]`) both land on the tail-index boundary.

## Dropped Candidates

- `35082e1` (TinyVec::fmt - fix pretty printing) — same bug class as a711c72 on TinyVec; keep ArrayVec
- `3335c63` (Fix TinyVec::resize across inline/heap) — pre-const-generic; resize surface has changed
- `48c004d` (Fix ArrayVec capacity u16::MAX) — requires u16::MAX-sized array, impractical
- `350cf62` (Fix Arbitrary implementation on ArrayVec) — API-only reshape, no observable bug
- `4cbd1db` (Fix element drop order on failed insert) — detection requires Drop tracker
- `95c991d` (Fix TinyVec::drain implementation) — same bug class as 9117614 on TinyVec; keep ArrayVec
- `f5de234` (heap allocated capacities) — TinyVec-only, no ArrayVec impact
- `be07a98` (fixed a truncate bug) — pre-const-generic API change
- `ed749ef` (SliceVec overflow on usize::MAX) — requires usize::MAX range, impractical
